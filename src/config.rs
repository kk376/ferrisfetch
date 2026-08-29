use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub modules: Option<Vec<String>>,
    pub disable: Option<Vec<String>>,
    pub logo: Option<String>,
    pub no_logo: Option<bool>,
    pub no_color: Option<bool>,
    pub disk_path: Option<String>,
    pub plugins: Option<Vec<PluginConfig>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginConfig {
    pub name: String,
    pub command: String,
}

impl Config {
    /// Loads configuration from default paths ($XDG_CONFIG_HOME/ferrisfetch/config.toml or ~/.config/ferrisfetch/config.toml).
    pub fn load_default() -> Self {
        if let Some(path) = get_default_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    return parse_config_toml(&content);
                }
            }
        }
        Config::default()
    }
}

pub fn get_default_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("ferrisfetch/config.toml"));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home).join(".config/ferrisfetch/config.toml"));
        }
    }
    None
}

/// Fast zero-dependency parser for ferrisfetch TOML config file.
pub fn parse_config_toml(content: &str) -> Config {
    let mut config = Config::default();
    let mut plugins = Vec::new();
    let mut in_plugin_section = false;
    let mut current_plugin_name = None;
    let mut current_plugin_command = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("[[plugins]]") || trimmed.starts_with("[[plugin]]") {
            if let (Some(n), Some(c)) = (current_plugin_name.take(), current_plugin_command.take()) {
                plugins.push(PluginConfig { name: n, command: c });
            }
            in_plugin_section = true;
            continue;
        } else if trimmed.starts_with('[') {
            if let (Some(n), Some(c)) = (current_plugin_name.take(), current_plugin_command.take()) {
                plugins.push(PluginConfig { name: n, command: c });
            }
            in_plugin_section = false;
            continue;
        }

        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim().to_lowercase();
            let val = v.trim();

            if in_plugin_section {
                let clean_val = val.trim_matches('"').trim_matches('\'').to_string();
                if key == "name" || key == "label" {
                    current_plugin_name = Some(clean_val);
                } else if key == "command" || key == "cmd" || key == "exec" {
                    current_plugin_command = Some(clean_val);
                }
                continue;
            }

            match key.as_str() {
                "modules" => {
                    config.modules = parse_toml_string_array(val);
                }
                "disable" => {
                    config.disable = parse_toml_string_array(val);
                }
                "logo" => {
                    let clean = val.trim_matches('"').trim_matches('\'').to_string();
                    if !clean.is_empty() && clean != "auto" {
                        config.logo = Some(clean);
                    }
                }
                "no_logo" => {
                    config.no_logo = Some(val.eq_ignore_ascii_case("true") || val == "1");
                }
                "no_color" => {
                    config.no_color = Some(val.eq_ignore_ascii_case("true") || val == "1");
                }
                "disk_path" => {
                    let clean = val.trim_matches('"').trim_matches('\'').to_string();
                    if !clean.is_empty() {
                        config.disk_path = Some(clean);
                    }
                }
                _ => {}
            }
        }
    }

    if let (Some(n), Some(c)) = (current_plugin_name.take(), current_plugin_command.take()) {
        plugins.push(PluginConfig { name: n, command: c });
    }

    if !plugins.is_empty() {
        config.plugins = Some(plugins);
    }

    config
}

pub fn parse_toml_string_array(val: &str) -> Option<Vec<String>> {
    let trimmed = val.trim();
    if !trimmed.starts_with('[') {
        return None;
    }
    let inner = trimmed.trim_start_matches('[').trim_end_matches(']');
    let items: Vec<String> = inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_toml_standard() {
        let toml = r#"
# Ferrisfetch config
modules = ["os", "kernel", "cpu", "memory"]
disable = ["gpu"]
logo = "fedora"
no_logo = false
no_color = false
disk_path = "/home"

[[plugins]]
name = "Spotify"
command = "playerctl metadata title"

[[plugins]]
name = "Git Branch"
command = "git branch --show-current"
"#;
        let config = parse_config_toml(toml);
        assert_eq!(
            config.modules,
            Some(vec![
                "os".to_string(),
                "kernel".to_string(),
                "cpu".to_string(),
                "memory".to_string(),
            ])
        );
        assert_eq!(config.disable, Some(vec!["gpu".to_string()]));
        assert_eq!(config.logo.as_deref(), Some("fedora"));
        assert_eq!(config.no_logo, Some(false));
        assert_eq!(config.disk_path.as_deref(), Some("/home"));

        let plugins = config.plugins.unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].name, "Spotify");
        assert_eq!(plugins[0].command, "playerctl metadata title");
        assert_eq!(plugins[1].name, "Git Branch");
        assert_eq!(plugins[1].command, "git branch --show-current");
    }
}
