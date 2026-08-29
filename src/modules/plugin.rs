use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
use std::process::Command;

pub struct PluginCollector;

impl Collector for PluginCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Plugin
    }

    fn collect_multiple(&self, ctx: &FetchContext) -> Vec<ModuleOutput> {
        let mut outputs = Vec::new();

        // 1. Plugins defined in config.toml
        if let Some(ref plugins) = ctx.config.plugins {
            for plugin in plugins {
                if let Some(out) = execute_plugin_command(&plugin.name, &plugin.command) {
                    outputs.push(out);
                }
            }
        }

        // 2. Executable scripts in ~/.config/ferrisfetch/plugins/
        if let Some(plugins_dir) = crate::config::get_default_config_path()
            .and_then(|p| p.parent().map(|d| d.join("plugins")))
        {
            if plugins_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(plugins_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
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
    let mut cmd = Command::new("sh");
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
}
