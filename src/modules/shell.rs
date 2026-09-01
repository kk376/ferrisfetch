use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;

/// Retrieves the parent process ID from `/proc/<pid>/status`.
#[cfg(not(windows))]
fn get_ppid(pid: u32) -> Option<u32> {
    let path = format!("/proc/{}/status", pid);
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("PPid:") {
            return rest.trim().parse::<u32>().ok();
        }
    }
    None
}

/// Retrieves the process command name from `/proc/<pid>/comm` or `/proc/<pid>/exe`.
#[cfg(not(windows))]
fn get_proc_name(pid: u32) -> Option<String> {
    let exe_path = format!("/proc/{}/exe", pid);
    if let Ok(target) = fs::read_link(exe_path) {
        if let Some(file_name) = target.file_name() {
            return Some(file_name.to_string_lossy().into_owned());
        }
    }

    let comm_path = format!("/proc/{}/comm", pid);
    if let Ok(comm) = fs::read_to_string(comm_path) {
        let clean = comm.trim().to_string();
        if !clean.is_empty() {
            return Some(clean);
        }
    }

    None
}

const KNOWN_SHELLS: &[&str] = &[
    "bash",
    "zsh",
    "fish",
    "sh",
    "dash",
    "ksh",
    "csh",
    "tcsh",
    "nu",
    "ion",
    "elvish",
    "pwsh",
    "powershell",
    "cmd",
];

/// Extracts and normalizes the shell name from a path or process command.
pub fn extract_shell_name(path: &str) -> String {
    let clean = path.trim();
    if clean.is_empty() {
        return String::new();
    }

    // Split on either '/' or '\' to ensure cross-platform path compatibility
    let file_name = clean.rsplit(['/', '\\']).next().unwrap_or(clean);

    // Login shells prepend a leading hyphen in argv[0] (e.g. "-bash" or "-zsh" per POSIX exec/login convention)
    let mut name = file_name.trim_start_matches('-').to_lowercase();
    if name.ends_with(".exe") {
        name.truncate(name.len() - 4);
    }
    name
}

/// Formats shell name with optional version information.
pub fn format_shell_name_version(
    shell_name: &str,
    bash_ver: Option<&str>,
    zsh_ver: Option<&str>,
    fish_ver: Option<&str>,
) -> String {
    let clean_name = shell_name.trim();

    if clean_name.contains("bash") {
        if let Some(ver) = bash_ver {
            let clean_ver = ver.split('(').next().unwrap_or(ver).trim();
            if !clean_ver.is_empty() {
                return format!("bash {}", clean_ver);
            }
        }
    } else if clean_name.contains("zsh") {
        if let Some(ver) = zsh_ver {
            let clean_ver = ver.trim();
            if !clean_ver.is_empty() {
                return format!("zsh {}", clean_ver);
            }
        }
    } else if clean_name.contains("fish") {
        if let Some(ver) = fish_ver {
            let clean_ver = ver.trim();
            if !clean_ver.is_empty() {
                return format!("fish {}", clean_ver);
            }
        }
    } else if clean_name == "powershell" || clean_name == "powershell.exe" {
        return "PowerShell 5.1".to_string();
    } else if clean_name == "cmd" || clean_name == "cmd.exe" {
        return "cmd.exe".to_string();
    }

    clean_name.to_string()
}

#[cfg(not(windows))]
fn get_shell_cli_version(shell_name: &str) -> Option<String> {
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("kkfetch"));

    let cache_file = cache_dir
        .as_ref()
        .map(|d| d.join(format!("shell_{}.cache", shell_name)));

    if let Some(ref path) = cache_file {
        if let Ok(cached) = std::fs::read_to_string(path) {
            let trimmed = cached.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    let output = crate::modules::system_command(shell_name)
        .arg("--version")
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        for word in text.split_whitespace() {
            let clean = word.trim_matches(',').trim_matches('(').trim_matches(')');
            if clean
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
                && clean.contains('.')
            {
                let ver = clean.split('(').next().unwrap_or(clean);
                let ver_str = ver.to_string();
                if let Some(ref dir) = cache_dir {
                    let _ = std::fs::create_dir_all(dir);
                }
                if let Some(ref path) = cache_file {
                    let _ = std::fs::write(path, &ver_str);
                }
                return Some(ver_str);
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn format_shell_with_version(shell_name: &str) -> String {
    // Fast-path: query shell version environment variables before spawning subprocesses
    let mut bash_ver = std::env::var("BASH_VERSION").ok();
    let mut zsh_ver = std::env::var("ZSH_VERSION").ok();
    let mut fish_ver = std::env::var("FISH_VERSION").ok();

    if bash_ver.is_none() && shell_name.contains("bash") {
        bash_ver = get_shell_cli_version("bash");
    }
    if zsh_ver.is_none() && shell_name.contains("zsh") {
        zsh_ver = get_shell_cli_version("zsh");
    }
    if fish_ver.is_none() && shell_name.contains("fish") {
        fish_ver = get_shell_cli_version("fish");
    }

    let res = format_shell_name_version(
        shell_name,
        bash_ver.as_deref(),
        zsh_ver.as_deref(),
        fish_ver.as_deref(),
    );

    if res == shell_name {
        if shell_name == "pwsh" {
            if let Ok(ver) = std::env::var("POWERSHELL_VERSION") {
                return format!("pwsh {}", ver.trim());
            }
            if let Some(cli_ver) = get_shell_cli_version("pwsh") {
                return format!("pwsh {}", cli_ver);
            }
        } else if shell_name == "nu" {
            if let Ok(ver) = std::env::var("NU_VERSION") {
                return format!("nu {}", ver.trim());
            }
            if let Some(cli_ver) = get_shell_cli_version("nu") {
                return format!("nu {}", cli_ver);
            }
        } else if shell_name == "cmd" {
            return "cmd.exe".to_string();
        } else if shell_name == "powershell" {
            if let Ok(ver) = std::env::var("PSVERSION") {
                return format!("PowerShell {}", ver.trim());
            }
            return "PowerShell 5.1".to_string();
        }

        if let Some(cli_ver) = get_shell_cli_version(shell_name) {
            return format!("{} {}", shell_name, cli_ver);
        }
    }

    res
}

/// Checks if a process name matches a known shell or valid versioned shell binary name.
/// Prevents false matches against unrelated daemon names (e.g. `shadow`, `shared-mime`, `shark`).
pub fn is_known_shell(name_clean: &str) -> bool {
    for &known in KNOWN_SHELLS {
        if name_clean == known {
            return true;
        }
        if let Some(rest) = name_clean.strip_prefix(known) {
            if rest.starts_with('-')
                || rest.starts_with('.')
                || (!rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
            {
                return true;
            }
        }
    }
    false
}

/// Pure helper to detect shell on Windows from environment variables.
pub fn detect_windows_shell_from_env(env_vars: &[(&str, &str)]) -> String {
    let get_env = |key: &str| {
        env_vars
            .iter()
            .find(|&&(k, _)| k.eq_ignore_ascii_case(key))
            .map(|&(_, v)| v.trim())
    };

    if let Some(shell_path) = get_env("SHELL") {
        let name_clean = extract_shell_name(shell_path);
        if !name_clean.is_empty() && is_known_shell(&name_clean) {
            return format_shell_name_version(&name_clean, None, None, None);
        }
    }

    if get_env("POWERSHELL_DISTRIBUTION_CHANNEL").is_some() {
        if let Some(ver) = get_env("POWERSHELL_VERSION") {
            return format!("pwsh {}", ver);
        }
        return "pwsh".to_string();
    }

    if let Some(ver) = get_env("NU_VERSION") {
        return format!("nu {}", ver);
    }

    if let Some(ps_mod) = get_env("PSModulePath") {
        if ps_mod.contains("PowerShell\\7")
            || ps_mod.contains("PowerShell/7")
            || ps_mod.to_lowercase().contains("pwsh")
        {
            if let Some(ver) = get_env("POWERSHELL_VERSION") {
                return format!("pwsh {}", ver);
            }
            return "pwsh".to_string();
        }
        if ps_mod.contains("WindowsPowerShell") {
            if let Some(ver) = get_env("PSVERSION") {
                return format!("PowerShell {}", ver);
            }
            return "PowerShell 5.1".to_string();
        }
    }

    if let Some(comspec) = get_env("COMSPEC") {
        let name = extract_shell_name(comspec);
        if name == "cmd" || is_known_shell(&name) {
            return "cmd.exe".to_string();
        }
    }

    if get_env("PROMPT").is_some() {
        return "cmd.exe".to_string();
    }

    "cmd.exe".to_string()
}

/// Probes process hierarchy or environment to determine active user shell.
#[cfg(windows)]
pub fn detect_shell() -> Option<String> {
    use crate::modules::win_util::ffi;

    // 1. Traverse process parent chain to identify the active interactive shell
    let chain = ffi::get_parent_process_chain(5);
    for (_pid, name) in &chain {
        let lower = name.to_lowercase();
        let clean = lower.trim_end_matches(".exe");

        if clean == "cmd" {
            let key = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
            let build = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "CurrentBuildNumber")
                .or_else(|| ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "CurrentBuild"));
            let ubr = ffi::reg_read_u32(ffi::HKEY_LOCAL_MACHINE, key, "UBR");

            if let Some(b) = build {
                if let Some(u) = ubr {
                    return Some(format!("CMD 10.0.{}.{}", b, u));
                }
                return Some(format!("CMD 10.0.{}", b));
            }
            return Some("cmd.exe".to_string());
        }

        if clean == "pwsh" {
            if let Ok(ver) = std::env::var("POWERSHELL_VERSION") {
                return Some(format!("pwsh {}", ver.trim()));
            }
            return Some("pwsh".to_string());
        }

        if clean == "powershell" {
            if let Ok(ver) = std::env::var("PSVERSION") {
                return Some(format!("PowerShell {}", ver.trim()));
            }
            return Some("PowerShell 5.1".to_string());
        }

        if clean == "nu" {
            if let Ok(ver) = std::env::var("NU_VERSION") {
                return Some(format!("nu {}", ver.trim()));
            }
            return Some("nu".to_string());
        }

        if is_known_shell(clean) {
            return Some(clean.to_string());
        }
    }

    // 2. Fallback to environment variables
    if let Ok(shell_path) = std::env::var("SHELL") {
        let name_clean = extract_shell_name(&shell_path);
        if !name_clean.is_empty() && is_known_shell(&name_clean) {
            return Some(name_clean);
        }
    }

    if let Ok(comspec) = std::env::var("COMSPEC") {
        let name = extract_shell_name(&comspec);
        if name == "cmd" || is_known_shell(&name) {
            return Some("cmd.exe".to_string());
        }
    }

    if std::env::var("PROMPT").is_ok() {
        return Some("cmd.exe".to_string());
    }

    Some("cmd.exe".to_string())
}

/// Probes process hierarchy or environment to determine active user shell.
#[cfg(not(windows))]
pub fn detect_shell() -> Option<String> {
    let mut current_pid = unsafe { libc::getpid() as u32 };

    // Traverse process parent chain (up to 5 ancestors) to identify the interactive shell that invoked kkfetch
    for _ in 0..5 {
        if let Some(ppid) = get_ppid(current_pid) {
            if ppid <= 1 {
                break;
            }
            if let Some(name) = get_proc_name(ppid) {
                let name_clean = extract_shell_name(&name);
                if is_known_shell(&name_clean) {
                    return Some(format_shell_with_version(&name_clean));
                }
            }
            current_pid = ppid;
        } else {
            break;
        }
    }

    // Fallback to $SHELL environment variable if parent process tree is masked or in a container
    if let Ok(shell_path) = std::env::var("SHELL") {
        let name_clean = extract_shell_name(&shell_path);
        if !name_clean.is_empty() {
            return Some(format_shell_with_version(&name_clean));
        }
    }

    // Check PowerShell Core or Nushell environment signatures on Unix
    if std::env::var("POWERSHELL_DISTRIBUTION_CHANNEL").is_ok() {
        return Some(format_shell_with_version("pwsh"));
    }
    if std::env::var("NU_VERSION").is_ok() {
        return Some(format_shell_with_version("nu"));
    }

    None
}

pub struct ShellCollector;

impl Collector for ShellCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Shell
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let shell = detect_shell()?;
        Some(ModuleOutput {
            id: ModuleId::Shell,
            label: "Shell".to_string(),
            value: shell,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_shell_name() {
        assert_eq!(extract_shell_name("/usr/bin/bash"), "bash");
        assert_eq!(extract_shell_name("/bin/-zsh"), "zsh");
        assert_eq!(extract_shell_name("/opt/homebrew/bin/fish"), "fish");
        assert_eq!(extract_shell_name("/usr/bin/nu"), "nu");
        assert_eq!(extract_shell_name("sh"), "sh");
        assert_eq!(extract_shell_name(""), "");

        // Windows shell paths with .exe
        assert_eq!(extract_shell_name("pwsh.exe"), "pwsh");
        assert_eq!(extract_shell_name("powershell.exe"), "powershell");
        assert_eq!(extract_shell_name("cmd.exe"), "cmd");
        assert_eq!(extract_shell_name("nu.exe"), "nu");
        assert_eq!(extract_shell_name("C:\\Windows\\System32\\cmd.exe"), "cmd");
        assert_eq!(
            extract_shell_name("C:\\Program Files\\PowerShell\\7\\pwsh.exe"),
            "pwsh"
        );
    }

    #[test]
    fn test_format_shell_name_version() {
        assert_eq!(
            format_shell_name_version("bash", Some("5.2.15(1)-release"), None, None),
            "bash 5.2.15"
        );
        assert_eq!(
            format_shell_name_version("zsh", None, Some("5.9"), None),
            "zsh 5.9"
        );
        assert_eq!(
            format_shell_name_version("fish", None, None, Some("3.7.0")),
            "fish 3.7.0"
        );
        assert_eq!(
            format_shell_name_version("powershell", None, None, None),
            "PowerShell 5.1"
        );
        assert_eq!(
            format_shell_name_version("cmd", None, None, None),
            "cmd.exe"
        );
        assert_eq!(
            format_shell_name_version("custom_shell", None, None, None),
            "custom_shell"
        );
    }

    #[test]
    fn test_is_known_shell() {
        assert!(is_known_shell("bash"));
        assert!(is_known_shell("zsh"));
        assert!(is_known_shell("fish"));
        assert!(is_known_shell("nu"));
        assert!(is_known_shell("sh"));
        assert!(is_known_shell("pwsh"));
        assert!(is_known_shell("powershell"));
        assert!(is_known_shell("cmd"));
        assert!(is_known_shell("bash-5.2"));
        assert!(is_known_shell("sh4"));

        // Reject non-shell prefixes
        assert!(!is_known_shell("shadow"));
        assert!(!is_known_shell("shared-mime"));
        assert!(!is_known_shell("nuget"));
        assert!(!is_known_shell("shark"));
        assert!(!is_known_shell("cmder"));
    }

    #[test]
    fn test_detect_windows_shell_from_env() {
        // 1. PowerShell 7 (pwsh)
        let pwsh_env = [
            ("POWERSHELL_DISTRIBUTION_CHANNEL", "MSI:Windows 10"),
            ("POWERSHELL_VERSION", "7.4.1"),
        ];
        assert_eq!(detect_windows_shell_from_env(&pwsh_env), "pwsh 7.4.1");

        // 2. Windows PowerShell 5.1
        let ps5_env = [(
            "PSModulePath",
            "C:\\Program Files\\WindowsPowerShell\\Modules;C:\\WINDOWS\\system32\\WindowsPowerShell\\v1.0\\Modules",
        )];
        assert_eq!(detect_windows_shell_from_env(&ps5_env), "PowerShell 5.1");

        // 3. Nushell
        let nu_env = [("NU_VERSION", "0.91.0")];
        assert_eq!(detect_windows_shell_from_env(&nu_env), "nu 0.91.0");

        // 4. Command Prompt
        let cmd_env = [
            ("COMSPEC", "C:\\Windows\\system32\\cmd.exe"),
            ("PROMPT", "$P$G"),
        ];
        assert_eq!(detect_windows_shell_from_env(&cmd_env), "cmd.exe");
    }
}
