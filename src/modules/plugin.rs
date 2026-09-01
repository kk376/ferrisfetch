use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

pub struct PluginCollector;

/// Checks whether kkfetch is running in an elevated/privileged context (e.g. sudo, su, setuid).
/// In elevated contexts, user-configured arbitrary plugins are intentionally disabled to prevent
/// privilege escalation via untrusted user configuration (F1).
#[cfg(unix)]
fn is_elevated_context() -> bool {
    unsafe {
        let uid = libc::getuid();
        let euid = libc::geteuid();
        if uid != euid || euid == 0 {
            return true;
        }
    }
    std::env::var("SUDO_USER").is_ok() || std::env::var("DOAS_USER").is_ok()
}

#[cfg(not(unix))]
fn is_elevated_context() -> bool {
    false
}

/// Checks whether a plugin file in ~/.config/kkfetch/plugins/ is safe and valid to execute (F2).
/// Requires:
/// 1. Regular file (not symlink or directory)
/// 2. Not a hidden file or editor backup (.swp, ~)
/// 3. Executable bit (+x) is set on Unix
/// 4. Owned by current user or root
fn is_safe_plugin_file(path: &Path) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    // Ignore hidden files, swap files, editor backups, or common non-executable extensions
    if file_name.starts_with('.') || file_name.ends_with('~') || file_name.ends_with(".swp") {
        return false;
    }

    #[cfg(unix)]
    {
        let meta = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        // Must be a regular file, not a symlink pointing elsewhere
        if !meta.file_type().is_file() {
            return false;
        }

        // Must have at least one executable permission bit set
        if meta.permissions().mode() & 0o111 == 0 {
            return false;
        }

        // Must be owned by current user or root
        let current_uid = unsafe { libc::getuid() };
        let file_uid = meta.uid();
        if file_uid != current_uid && file_uid != 0 {
            return false;
        }
    }

    #[cfg(not(unix))]
    {
        if !path.is_file() {
            return false;
        }
    }

    true
}

impl Collector for PluginCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Plugin
    }

    fn collect_multiple(&self, ctx: &FetchContext) -> Vec<ModuleOutput> {
        // Prevent arbitrary code execution when running with elevated privileges (F1)
        if is_elevated_context() {
            return Vec::new();
        }

        let mut outputs = Vec::new();

        // 1. Plugins defined in config.toml
        if let Some(ref plugins) = ctx.config.plugins {
            for plugin in plugins {
                if let Some(out) = execute_plugin_command(&plugin.name, &plugin.command) {
                    outputs.push(out);
                }
            }
        }

        // 2. Executable scripts in ~/.config/kkfetch/plugins/ (F2)
        if let Some(plugins_dir) = crate::config::get_default_config_path()
            .and_then(|p| p.parent().map(|d| d.join("plugins")))
        {
            if plugins_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(plugins_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if is_safe_plugin_file(&path) {
                            let name = path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            let cmd = path.to_string_lossy().to_string();
                            if let Some(out) = execute_plugin_command(&name, &cmd) {
                                outputs.push(out);
                            }
                        }
                    }
                }
            }
        }

        outputs
    }
}

pub fn execute_plugin_command(name: &str, command: &str) -> Option<ModuleOutput> {
    #[cfg(unix)]
    let mut cmd = Command::new("/bin/sh");
    #[cfg(unix)]
    cmd.arg("-c").arg(command);

    #[cfg(windows)]
    let mut cmd = Command::new("cmd");
    #[cfg(windows)]
    cmd.arg("/C").arg(command);

    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let clean = stdout.trim();
            if !clean.is_empty() {
                return Some(ModuleOutput {
                    id: ModuleId::Plugin,
                    label: name.to_string(),
                    value: clean.to_string(),
                    custom_rendered: None,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_plugin_command_echo() {
        let out = execute_plugin_command("TestEcho", "echo HelloFerris").unwrap();
        assert_eq!(out.label, "TestEcho");
        assert_eq!(out.value, "HelloFerris");
        assert_eq!(out.id, ModuleId::Plugin);
    }

    #[test]
    fn test_is_safe_plugin_file_filters_invalid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let regular_txt = temp_dir.path().join("readme.txt");
        std::fs::write(&regular_txt, "hello").unwrap();

        #[cfg(unix)]
        {
            // Non-executable should be rejected
            assert!(!is_safe_plugin_file(&regular_txt));

            // Executable should be accepted
            let mut perms = std::fs::metadata(&regular_txt).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&regular_txt, perms).unwrap();
            assert!(is_safe_plugin_file(&regular_txt));
        }

        // Hidden files should be rejected
        let hidden = temp_dir.path().join(".hidden_script");
        std::fs::write(&hidden, "echo hidden").unwrap();
        assert!(!is_safe_plugin_file(&hidden));
    }
}
