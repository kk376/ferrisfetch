use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;

const KNOWN_TERMINALS: &[(&str, &str)] = &[
    ("ptyxis-agent", "Ptyxis"),
    ("ptyxis", "Ptyxis"),
    ("gnome-terminal-server", "GNOME Terminal"),
    ("gnome-terminal", "GNOME Terminal"),
    ("gnome-console", "GNOME Console"),
    ("kgx", "GNOME Console"),
    ("konsole", "Konsole"),
    ("alacritty", "Alacritty"),
    ("kitty", "kitty"),
    ("ghostty", "Ghostty"),
    ("wezterm-gui", "WezTerm"),
    ("wezterm", "WezTerm"),
    ("foot", "foot"),
    ("rio", "Rio"),
    ("contour", "Contour"),
    ("blackbox", "BlackBox"),
    ("xterm", "xterm"),
    ("urxvt", "urxvt"),
    ("rxvt", "rxvt"),
    ("st", "st"),
    ("terminator", "Terminator"),
    ("xfce4-terminal", "XFCE Terminal"),
    ("mate-terminal", "MATE Terminal"),
    ("lxterminal", "LXTerminal"),
    ("qterminal", "QTerminal"),
    ("tilix", "Tilix"),
    ("guake", "Guake"),
    ("yakuake", "Yakuake"),
    ("tilda", "Tilda"),
    ("sakura", "Sakura"),
    ("termite", "Termite"),
    ("tabby", "Tabby"),
    ("hyper", "Hyper"),
    ("warp", "Warp"),
    ("deepin-terminal", "Deepin Terminal"),
    ("pantheon-terminal", "Pantheon Terminal"),
    ("io.elementary.terminal", "Pantheon Terminal"),
    ("zellij", "Zellij"),
    ("tmux", "tmux"),
];

/// Inspects environment variables to detect terminal emulator.
pub fn detect_terminal_from_env(
    term_program: Option<&str>,
    term_program_version: Option<&str>,
    env_vars: &[(&str, &str)],
    term: Option<&str>,
) -> Option<String> {
    // 1. Check $TERM_PROGRAM
    if let Some(prog) = term_program {
        let clean_prog = prog.trim();
        if clean_prog.eq_ignore_ascii_case("vscode") || clean_prog == "Code" {
            if let Some(ver) = term_program_version {
                let clean_ver = ver.trim();
                if !clean_ver.is_empty() {
                    return Some(format!("Visual Studio Code {}", clean_ver));
                }
            }
            return Some("Visual Studio Code".to_string());
        } else if !clean_prog.is_empty() {
            if let Some(ver) = term_program_version {
                let clean_ver = ver.trim();
                if !clean_ver.is_empty() {
                    return Some(format!("{} {}", clean_prog, clean_ver));
                }
            }
            return Some(clean_prog.to_string());
        }
    }

    // 2. Check dedicated terminal environment signatures
    let has_env = |var_name: &str| env_vars.iter().any(|&(k, _)| k == var_name);
    let get_env_val = |var_name: &str| {
        env_vars
            .iter()
            .find(|&&(k, _)| k == var_name)
            .map(|&(_, v)| v.trim())
    };

    if has_env("VSCODE_INJECTION") {
        return Some("Visual Studio Code".to_string());
    }

    if let Some(ver) = get_env_val("PTYXIS_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Ptyxis {}", ver));
        }
        return Some("Ptyxis".to_string());
    }

    if let Some(ver) = get_env_val("GHOSTTY_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Ghostty {}", ver));
        }
        return Some("Ghostty".to_string());
    }
    if has_env("GHOSTTY_RESOURCES_DIR") {
        return Some("Ghostty".to_string());
    }

    if let Some(ver) = get_env_val("KGX_VERSION") {
        if !ver.is_empty() {
            return Some(format!("GNOME Console {}", ver));
        }
        return Some("GNOME Console".to_string());
    }

    if has_env("ALACRITTY_LOG") || has_env("ALACRITTY_WINDOW_ID") || has_env("ALACRITTY_SOCKET") {
        return Some("Alacritty".to_string());
    }

    if has_env("KITTY_PID") || has_env("KITTY_WINDOW_ID") {
        return Some("kitty".to_string());
    }

    if let Some(ver) = get_env_val("KONSOLE_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Konsole {}", ver));
        }
        return Some("Konsole".to_string());
    }

    if has_env("WT_SESSION") {
        return Some("Windows Terminal".to_string());
    }

    if let Some(ver) = get_env_val("CONTOUR_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Contour {}", ver));
        }
        return Some("Contour".to_string());
    }

    if let Some(ver) = get_env_val("RIO_VERSION") {
        if !ver.is_empty() {
            return Some(format!("Rio {}", ver));
        }
        return Some("Rio".to_string());
    }

    if let Some(ver) = get_env_val("BLACKBOX_VERSION") {
        if !ver.is_empty() {
            return Some(format!("BlackBox {}", ver));
        }
        return Some("BlackBox".to_string());
    }

    if has_env("TERMINOLOGY") {
        return Some("Terminology".to_string());
    }

    if let Some(ver) = get_env_val("XTERM_VERSION") {
        if !ver.is_empty() {
            return Some(format!("xterm {}", ver));
        }
        return Some("xterm".to_string());
    }

    if has_env("GNOME_TERMINAL_SCREEN") || has_env("GNOME_TERMINAL_SERVICE") {
        return Some("GNOME Terminal".to_string());
    }

    if has_env("MATE_TERMINAL_SCREEN") {
        return Some("MATE Terminal".to_string());
    }

    if has_env("TILIX_ID") {
        return Some("Tilix".to_string());
    }

    if has_env("WEZTERM_PANE") {
        return Some("WezTerm".to_string());
    }

    if has_env("WARP_IS_LOCAL_SHELL_SESSION") {
        return Some("Warp".to_string());
    }

    if has_env("ZELLIJ") || has_env("ZELLIJ_SESSION_NAME") {
        return Some("Zellij".to_string());
    }

    if has_env("FOOT_PID") {
        return Some("foot".to_string());
    }

    // 3. Fallback to $TERM
    if let Some(t) = term {
        let clean = t.trim();
        if !clean.is_empty() && clean != "unknown" && clean != "dumb" {
            return Some(clean.to_string());
        }
    }

    None
}

pub fn match_terminal_proc(comm: &str) -> Option<&'static str> {
    for &(proc_name, display_name) in KNOWN_TERMINALS {
        let is_match = if proc_name == "st" {
            comm == "st" || comm == "stterm" || comm.starts_with("st-")
        } else {
            comm == proc_name || comm.starts_with(&format!("{}-", proc_name))
        };
        if is_match {
            return Some(display_name);
        }
    }
    None
}

/// Pure helper to detect terminal on Windows from environment variables.
pub fn detect_windows_terminal_from_env(
    term_program: Option<&str>,
    term_program_version: Option<&str>,
    env_vars: &[(&str, &str)],
) -> String {
    detect_terminal_from_env(term_program, term_program_version, env_vars, None)
        .unwrap_or_else(|| "Console Window Host (ConHost)".to_string())
}

/// Inspects environment variables and process ancestry to detect terminal emulator.
#[cfg(windows)]
pub fn detect_terminal() -> Option<String> {
    use crate::modules::win_util::ffi;

    let term_prog = std::env::var("TERM_PROGRAM").ok();
    let term_prog_ver = std::env::var("TERM_PROGRAM_VERSION").ok();

    let env_signatures = [
        "WT_SESSION",
        "VSCODE_INJECTION",
        "ALACRITTY_LOG",
        "ALACRITTY_WINDOW_ID",
        "ALACRITTY_SOCKET",
        "WEZTERM_PANE",
        "KONSOLE_VERSION",
        "GHOSTTY_VERSION",
    ];

    let mut present_vars = Vec::new();
    for &sig in &env_signatures {
        if let Ok(val) = std::env::var(sig) {
            present_vars.push((sig, val));
        }
    }
    let ref_vars: Vec<(&str, &str)> = present_vars.iter().map(|(k, v)| (*k, v.as_str())).collect();

    if let Some(term) = detect_terminal_from_env(
        term_prog.as_deref(),
        term_prog_ver.as_deref(),
        &ref_vars,
        None,
    ) {
        return Some(term);
    }

    // Check parent process chain for terminal emulator
    let chain = ffi::get_parent_process_chain(5);
    for (_pid, name) in &chain {
        let lower = name.to_lowercase();
        if lower.contains("windowsterminal") {
            return Some("Windows Terminal".to_string());
        }
        if lower.contains("alacritty") {
            return Some("Alacritty".to_string());
        }
        if lower.contains("wezterm") {
            return Some("WezTerm".to_string());
        }
        if lower.contains("code") {
            return Some("Visual Studio Code".to_string());
        }
        if lower.contains("mintty") {
            return Some("MinTTY".to_string());
        }
    }

    Some("Console Window Host (ConHost)".to_string())
}

/// Extracts the first version-like token from terminal version stdout/stderr.
pub fn extract_terminal_version_from_output(output: &str) -> Option<String> {
    for line in output.lines() {
        for word in line.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_');
            if let Some(first) = clean.chars().next() {
                if first.is_ascii_digit() && (clean.contains('.') || clean.len() >= 6) {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

/// Appends version to terminal name if not already present.
pub fn append_version_if_missing(term_display_name: &str, version: Option<&str>) -> String {
    let Some(ver) = version else {
        return term_display_name.to_string();
    };
    let clean_ver = ver.trim();
    if clean_ver.is_empty() {
        return term_display_name.to_string();
    }
    if term_display_name.contains(clean_ver) {
        return term_display_name.to_string();
    }
    format!("{} {}", term_display_name, clean_ver)
}

#[cfg(not(windows))]
fn get_terminal_cache_path(binary: &str) -> std::path::PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = std::path::Path::new(&runtime_dir);
        if dir.is_dir() {
            return dir.join(format!("ferrisfetch_term_{}.cache", binary));
        }
    }
    let uid = unsafe { libc::getuid() };
    std::path::PathBuf::from(format!("/tmp/ferrisfetch_term_{}_{}.cache", binary, uid))
}

#[cfg(not(windows))]
fn get_binary_mtime(binary: &str) -> Option<u64> {
    let standard_paths = [
        format!("/usr/bin/{}", binary),
        format!("/usr/local/bin/{}", binary),
        format!("/bin/{}", binary),
    ];

    for p in &standard_paths {
        let path = std::path::Path::new(p);
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                    return Some(dur.as_secs());
                }
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let local_bin = std::path::Path::new(&home).join(format!(".local/bin/{}", binary));
        if let Ok(meta) = fs::metadata(&local_bin) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                    return Some(dur.as_secs());
                }
            }
        }
    }

    None
}

#[cfg(not(windows))]
pub fn probe_terminal_cli_version(term_name: &str) -> Option<String> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, Option<String>>>,
    > = std::sync::OnceLock::new();
    let cache_mutex = CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));

    if let Ok(guard) = cache_mutex.lock() {
        if let Some(cached) = guard.get(term_name) {
            return cached.clone();
        }
    }

    let result = probe_terminal_cli_version_uncached(term_name);

    if let Ok(mut guard) = cache_mutex.lock() {
        guard.insert(term_name.to_string(), result.clone());
    }

    result
}

#[cfg(not(windows))]
fn probe_terminal_cli_version_uncached(term_name: &str) -> Option<String> {
    let lower = term_name.to_lowercase();
    let (binary, args): (&str, &[&str]) = if lower.contains("kitty") {
        ("kitty", &["--version"])
    } else if lower.contains("alacritty") {
        ("alacritty", &["--version"])
    } else if lower.contains("foot") {
        ("foot", &["--version"])
    } else if lower.contains("wezterm") {
        ("wezterm", &["--version"])
    } else if lower.contains("ghostty") {
        ("ghostty", &["--version"])
    } else if lower.contains("gnome-terminal") || lower == "gnome terminal" {
        ("gnome-terminal", &["--version"])
    } else if lower.contains("gnome-console") || lower == "gnome console" || lower == "kgx" {
        ("kgx", &["--version"])
    } else if lower.contains("konsole") {
        ("konsole", &["--version"])
    } else if lower.contains("xfce4-terminal") || lower == "xfce terminal" {
        ("xfce4-terminal", &["--version"])
    } else if lower.contains("mate-terminal") || lower == "mate terminal" {
        ("mate-terminal", &["--version"])
    } else if lower.contains("tilix") {
        ("tilix", &["--version"])
    } else if lower.contains("terminator") {
        ("terminator", &["--version"])
    } else if lower == "tmux" {
        ("tmux", &["-V"])
    } else if lower == "zellij" {
        ("zellij", &["--version"])
    } else if lower == "rio" {
        ("rio", &["--version"])
    } else if lower == "contour" {
        ("contour", &["--version"])
    } else if lower == "blackbox" {
        ("blackbox", &["--version"])
    } else if lower == "ptyxis" {
        ("ptyxis", &["--version"])
    } else if lower == "xterm" {
        ("xterm", &["-version"])
    } else {
        return None;
    };

    let cache_path = get_terminal_cache_path(binary);
    let bin_mtime = get_binary_mtime(binary);

    // 1. Try reading from mtime-validated tmpfs runtime cache (< 2 µs)
    if let Some(current_mtime) = bin_mtime {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Some((cached_mtime_str, ver)) = content.split_once('|') {
                if let Ok(cached_mtime) = cached_mtime_str.parse::<u64>() {
                    if cached_mtime == current_mtime && !ver.trim().is_empty() {
                        return Some(ver.trim().to_string());
                    }
                }
            }
        }
    }

    // 2. Cache miss: execute subprocess once and persist to tmpfs
    if let Ok(output) = std::process::Command::new(binary).args(args).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let ver = if let Some(v) = extract_terminal_version_from_output(&stdout) {
            Some(v)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            extract_terminal_version_from_output(&stderr)
        };

        if let Some(ref version_str) = ver {
            if let Some(current_mtime) = bin_mtime {
                let _ = fs::write(&cache_path, format!("{}|{}", current_mtime, version_str));
            }
        }

        return ver;
    }
    None
}

/// Inspects environment variables and process ancestry to detect terminal emulator.
#[cfg(not(windows))]
pub fn detect_terminal() -> Option<String> {
    let term_prog = std::env::var("TERM_PROGRAM").ok();
    let term_prog_ver = std::env::var("TERM_PROGRAM_VERSION").ok();
    let term_val = std::env::var("TERM").ok();

    // 1. Check dedicated terminal emulator environment signatures
    let env_signatures = [
        "ALACRITTY_LOG",
        "ALACRITTY_WINDOW_ID",
        "ALACRITTY_SOCKET",
        "KITTY_PID",
        "KITTY_WINDOW_ID",
        "KONSOLE_VERSION",
        "WT_SESSION",
        "VSCODE_INJECTION",
        "FOOT_PID",
        "TERMINOLOGY",
        "XTERM_VERSION",
        "GNOME_TERMINAL_SCREEN",
        "GNOME_TERMINAL_SERVICE",
        "TILIX_ID",
        "WEZTERM_PANE",
    ];

    let mut present_vars = Vec::new();
    for &sig in &env_signatures {
        if let Ok(val) = std::env::var(sig) {
            present_vars.push((sig, val));
        }
    }
    let ref_vars: Vec<(&str, &str)> = present_vars.iter().map(|(k, v)| (*k, v.as_str())).collect();

    if let Some(term) = detect_terminal_from_env(
        term_prog.as_deref(),
        term_prog_ver.as_deref(),
        &ref_vars,
        None, // Defer generic $TERM fallback until process ancestry is checked
    ) {
        let ver = probe_terminal_cli_version(&term);
        return Some(append_version_if_missing(&term, ver.as_deref()));
    }

    // 2. Process ancestry traversal: walk up to 8 levels of PPID to jump over subshells, tmux/screen, and sudo wrappers
    let mut current_pid = unsafe { libc::getpid() as u32 };
    for _ in 0..8 {
        let status_path = format!("/proc/{}/status", current_pid);
        let ppid = if let Ok(status) = fs::read_to_string(status_path) {
            status.lines().find_map(|l| {
                l.strip_prefix("PPid:")
                    .and_then(|p| p.trim().parse::<u32>().ok())
            })
        } else {
            None
        };

        if let Some(ppid) = ppid {
            if ppid <= 1 {
                break;
            }

            let comm = fs::read_to_string(format!("/proc/{}/comm", ppid))
                .unwrap_or_default()
                .trim()
                .to_lowercase();

            if let Some(display_name) = match_terminal_proc(&comm) {
                let ver = probe_terminal_cli_version(display_name);
                return Some(append_version_if_missing(display_name, ver.as_deref()));
            }

            current_pid = ppid;
        } else {
            break;
        }
    }

    // 3. Fallback to generic $TERM string if specific GUI terminal binary was not found
    if let Some(term) = term_val {
        let clean = term.trim();
        if !clean.is_empty() && clean != "unknown" && clean != "dumb" {
            return Some(clean.to_string());
        }
    }

    None
}

pub struct TerminalCollector;

impl Collector for TerminalCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Terminal
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let term = detect_terminal()?;
        Some(ModuleOutput {
            id: ModuleId::Terminal,
            label: "Terminal".to_string(),
            value: term,
            custom_rendered: None,
        })
    }
}

/// Probes the active terminal font configuration from local user dotfiles or system settings.
#[cfg(not(windows))]
pub fn detect_terminal_font() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let home_path = std::path::Path::new(&home);

    // 1. Kitty
    let kitty_conf = home_path.join(".config/kitty/kitty.conf");
    if let Ok(content) = fs::read_to_string(kitty_conf) {
        let mut family = None;
        let mut size = None;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("font_family") {
                let f = rest.trim();
                if !f.is_empty() && f != "auto" {
                    family = Some(f.to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("font_size") {
                let s = rest.trim();
                if !s.is_empty() {
                    size = Some(s.to_string());
                }
            }
        }
        if let Some(f) = family {
            if let Some(s) = size {
                return Some(format!("{} ({}pt)", f, s));
            }
            return Some(f);
        }
    }

    // 2. Alacritty
    for path in &[
        home_path.join(".config/alacritty/alacritty.toml"),
        home_path.join(".alacritty.toml"),
    ] {
        if let Ok(content) = fs::read_to_string(path) {
            let mut family = None;
            let mut size = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if k == "family" && family.is_none() {
                        family = Some(v.to_string());
                    } else if k == "size" && size.is_none() {
                        size = Some(v.to_string());
                    }
                }
            }
            if let Some(f) = family {
                if let Some(s) = size {
                    return Some(format!("{} ({}pt)", f, s));
                }
                return Some(f);
            }
        }
    }

    // 3. Foot
    let foot_ini = home_path.join(".config/foot/foot.ini");
    if let Ok(content) = fs::read_to_string(foot_ini) {
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("font=") {
                let f = rest.trim();
                if !f.is_empty() {
                    return Some(f.to_string());
                }
            }
        }
    }

    None
}

#[cfg(windows)]
pub fn detect_terminal_font() -> Option<String> {
    Some("Consolas (11pt)".to_string())
}

pub struct TerminalFontCollector;

impl Collector for TerminalFontCollector {
    fn id(&self) -> ModuleId {
        ModuleId::TerminalFont
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let font = detect_terminal_font()?;
        Some(ModuleOutput {
            id: ModuleId::TerminalFont,
            label: "Terminal Font".to_string(),
            value: font,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_terminal_from_env_term_program() {
        let res =
            detect_terminal_from_env(Some("WezTerm"), Some("20240203-110809-5046fc22"), &[], None);
        assert_eq!(res, Some("WezTerm 20240203-110809-5046fc22".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_alacritty() {
        let envs = [("ALACRITTY_SOCKET", "/run/user/1000/alacritty.sock")];
        let res = detect_terminal_from_env(None, None, &envs, None);
        assert_eq!(res, Some("Alacritty".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_kitty() {
        let envs = [("KITTY_PID", "12345")];
        let res = detect_terminal_from_env(None, None, &envs, None);
        assert_eq!(res, Some("kitty".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_konsole() {
        let envs = [("KONSOLE_VERSION", "230805")];
        let res = detect_terminal_from_env(None, None, &envs, None);
        assert_eq!(res, Some("Konsole 230805".to_string()));
    }

    #[test]
    fn test_detect_terminal_from_env_fallback_term() {
        let res = detect_terminal_from_env(None, None, &[], Some("xterm-256color"));
        assert_eq!(res, Some("xterm-256color".to_string()));

        let res_unknown = detect_terminal_from_env(None, None, &[], Some("unknown"));
        assert_eq!(res_unknown, None);

        let res_none = detect_terminal_from_env(None, None, &[], None);
        assert_eq!(res_none, None);
    }

    #[test]
    fn test_detect_windows_terminals() {
        // 1. Windows Terminal
        let wt_env = [("WT_SESSION", "3f2e1a0b-1234-5678-9abc-def012345678")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &wt_env),
            "Windows Terminal"
        );

        // 2. Visual Studio Code via TERM_PROGRAM
        assert_eq!(
            detect_windows_terminal_from_env(Some("vscode"), Some("1.87.2"), &[]),
            "Visual Studio Code 1.87.2"
        );
        assert_eq!(
            detect_windows_terminal_from_env(Some("Code"), None, &[]),
            "Visual Studio Code"
        );

        // 3. Visual Studio Code via VSCODE_INJECTION
        let vscode_inj = [("VSCODE_INJECTION", "1")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &vscode_inj),
            "Visual Studio Code"
        );

        // 4. Alacritty
        let alacritty_env = [("ALACRITTY_LOG", "C:\\alacritty.log")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &alacritty_env),
            "Alacritty"
        );

        // 5. WezTerm
        let wezterm_env = [("WEZTERM_PANE", "0")];
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &wezterm_env),
            "WezTerm"
        );

        // 6. ConHost fallback
        assert_eq!(
            detect_windows_terminal_from_env(None, None, &[]),
            "Console Window Host (ConHost)"
        );
    }

    #[test]
    fn test_match_terminal_proc() {
        assert_eq!(match_terminal_proc("st"), Some("st"));
        assert_eq!(match_terminal_proc("stterm"), Some("st"));
        assert_eq!(match_terminal_proc("st-256color"), Some("st"));
        assert_eq!(match_terminal_proc("alacritty"), Some("Alacritty"));
        assert_eq!(match_terminal_proc("kitty"), Some("kitty"));
        assert_eq!(
            match_terminal_proc("gnome-terminal-server"),
            Some("GNOME Terminal")
        );

        // Ensure false-positive substrings do not match
        assert_eq!(match_terminal_proc("systemd"), None);
        assert_eq!(match_terminal_proc("starship"), None);
        assert_eq!(match_terminal_proc("strace"), None);
        assert_eq!(match_terminal_proc("install"), None);
        assert_eq!(match_terminal_proc("gst-plugin"), None);
    }

    #[test]
    fn test_extract_terminal_version_from_output() {
        assert_eq!(
            extract_terminal_version_from_output("kitty 0.48.2 created by Kovid Goyal").as_deref(),
            Some("0.48.2")
        );
        assert_eq!(
            extract_terminal_version_from_output("alacritty 0.17.0").as_deref(),
            Some("0.17.0")
        );
        assert_eq!(
            extract_terminal_version_from_output("foot version: 1.16.2").as_deref(),
            Some("1.16.2")
        );
        assert_eq!(
            extract_terminal_version_from_output("wezterm 20240203-110809-5046fc22").as_deref(),
            Some("20240203-110809-5046fc22")
        );
        assert_eq!(
            extract_terminal_version_from_output("GNOME Terminal 3.50.1 using VTE 0.74.0 +BFD")
                .as_deref(),
            Some("3.50.1")
        );
        assert_eq!(
            extract_terminal_version_from_output("tmux 3.4").as_deref(),
            Some("3.4")
        );
        assert_eq!(
            extract_terminal_version_from_output("invalid output without version"),
            None
        );
    }

    #[test]
    fn test_append_version_if_missing() {
        assert_eq!(
            append_version_if_missing("kitty", Some("0.48.2")),
            "kitty 0.48.2"
        );
        assert_eq!(
            append_version_if_missing("Ptyxis 47.0", Some("47.0")),
            "Ptyxis 47.0"
        );
        assert_eq!(append_version_if_missing("Alacritty", None), "Alacritty");
    }
}
