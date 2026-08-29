#[cfg(not(windows))]
use std::path::{Path, PathBuf};

use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThemeInfo {
    pub theme: Option<String>,
    pub icon_theme: Option<String>,
    pub font: Option<String>,
    pub cursor: Option<String>,
    pub cursor_size: Option<u32>,
    pub dark_mode: bool,
    pub source: Option<ThemeSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSource {
    Gtk,
    Kde,
    Xfce,
    GSettings,
    Env,
    Windows,
}

impl ThemeSource {
    pub fn label_suffix(&self) -> &'static str {
        match self {
            ThemeSource::Gtk => "[GTK]",
            ThemeSource::Kde => "[Qt/KDE]",
            ThemeSource::Xfce => "[XFCE]",
            ThemeSource::GSettings => "[GTK/GNOME]",
            ThemeSource::Env => "[Env]",
            ThemeSource::Windows => "[Windows]",
        }
    }
}

/// Parses GTK 3.0 or 4.0 `settings.ini` file contents.
pub fn parse_gtk_settings_ini(content: &str) -> ThemeInfo {
    let mut info = ThemeInfo {
        source: Some(ThemeSource::Gtk),
        ..Default::default()
    };

    let mut in_settings_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_settings_section = trimmed.eq_ignore_ascii_case("[Settings]");
            continue;
        }

        if !in_settings_section {
            continue;
        }

        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim().to_ascii_lowercase();
            let val = val.trim().trim_matches('"').trim_matches('\'').to_string();

            if val.is_empty() {
                continue;
            }

            match key.as_str() {
                "gtk-theme-name" => info.theme = Some(val),
                "gtk-icon-theme-name" => info.icon_theme = Some(val),
                "gtk-font-name" => info.font = Some(val),
                "gtk-cursor-theme-name" => info.cursor = Some(val),
                "gtk-cursor-theme-size" => {
                    if let Ok(size) = val.parse::<u32>() {
                        info.cursor_size = Some(size);
                    }
                }
                "gtk-application-prefer-dark-theme" => {
                    info.dark_mode = val == "1" || val.eq_ignore_ascii_case("true");
                }
                _ => {}
            }
        }
    }

    info
}

/// Parses KDE Plasma `~/.config/kdeglobals` configuration.
pub fn parse_kde_globals(content: &str) -> ThemeInfo {
    let mut info = ThemeInfo {
        source: Some(ThemeSource::Kde),
        ..Default::default()
    };

    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_ascii_lowercase();
            continue;
        }

        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim().to_ascii_lowercase();
            let val = val.trim().trim_matches('"').trim_matches('\'').to_string();

            if val.is_empty() {
                continue;
            }

            match (current_section.as_str(), key.as_str()) {
                ("kde", "lookandfeelpackage") => {
                    let name = val
                        .trim_start_matches("org.kde.")
                        .trim_end_matches(".desktop")
                        .replace("dark", "-Dark")
                        .replace("light", "-Light");
                    info.theme = Some(name);
                }
                ("kde", "widgetstyle") if info.theme.is_none() => {
                    info.theme = Some(val);
                }
                ("general", "colorscheme") => {
                    if val.to_ascii_lowercase().contains("dark") {
                        info.dark_mode = true;
                    }
                    if info.theme.is_none() {
                        info.theme = Some(val);
                    }
                }
                ("icons", "theme") => {
                    info.icon_theme = Some(val);
                }
                _ => {}
            }
        }
    }

    info
}

/// Parses XFCE `xsettings.xml` file.
pub fn parse_xfce_xsettings(content: &str) -> ThemeInfo {
    let mut info = ThemeInfo {
        source: Some(ThemeSource::Xfce),
        ..Default::default()
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("name=\"ThemeName\"") {
            if let Some(val) = extract_xml_property_value(trimmed) {
                info.theme = Some(val);
            }
        } else if trimmed.contains("name=\"IconThemeName\"") {
            if let Some(val) = extract_xml_property_value(trimmed) {
                info.icon_theme = Some(val);
            }
        }
    }

    info
}

fn extract_xml_property_value(line: &str) -> Option<String> {
    if let Some(val_idx) = line.find("value=\"") {
        let remainder = &line[val_idx + 7..];
        if let Some(end_quote) = remainder.find('"') {
            let val = remainder[..end_quote].trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Attempts to query GSettings for GNOME/GTK theme and icon properties.
#[cfg(not(windows))]
fn query_gsettings_theme() -> ThemeInfo {
    let mut info = ThemeInfo {
        source: Some(ThemeSource::GSettings),
        ..Default::default()
    };

    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none()
        && std::env::var_os("DISPLAY").is_none()
        && std::env::var_os("WAYLAND_DISPLAY").is_none()
    {
        return info;
    }

    // Fast path: Query all interface properties in a single dconf dump call (12ms) instead of 3 separate gsettings calls (36ms)
    if let Ok(output) = crate::modules::system_command("dconf")
        .args(["dump", "/org/gnome/desktop/interface/"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('\'').trim_matches('"');
                    match k {
                        "gtk-theme" => info.theme = Some(v.to_string()),
                        "icon-theme" => info.icon_theme = Some(v.to_string()),
                        "font-name" => info.font = Some(v.to_string()),
                        "cursor-theme" => info.cursor = Some(v.to_string()),
                        "cursor-size" => {
                            if let Ok(s) = v.parse::<u32>() {
                                info.cursor_size = Some(s);
                            }
                        }
                        "color-scheme" if v.to_ascii_lowercase().contains("dark") => {
                            info.dark_mode = true;
                        }
                        _ => {}
                    }
                }
            }
            if info.theme.is_some() || info.icon_theme.is_some() || info.dark_mode {
                if info.theme.is_none() {
                    info.theme = Some("Adwaita".to_string());
                }
                if info.icon_theme.is_none() {
                    info.icon_theme = Some("Adwaita".to_string());
                }
                if info.cursor.is_none() {
                    info.cursor = Some("Adwaita".to_string());
                    if info.cursor_size.is_none() {
                        info.cursor_size = Some(24);
                    }
                }
                return info;
            }
        }
    }

    // Fallback: Individual GSettings queries
    if let Ok(output) = crate::modules::system_command("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
    {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if !val.is_empty() {
                info.theme = Some(val);
            }
        }
    }

    if let Ok(output) = crate::modules::system_command("gsettings")
        .args(["get", "org.gnome.desktop.interface", "icon-theme"])
        .output()
    {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if !val.is_empty() {
                info.icon_theme = Some(val);
            }
        }
    }

    if let Ok(output) = crate::modules::system_command("gsettings")
        .args(["get", "org.gnome.desktop.interface", "cursor-theme"])
        .output()
    {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if !val.is_empty() {
                info.cursor = Some(val);
            }
        }
    }

    if let Ok(output) = crate::modules::system_command("gsettings")
        .args(["get", "org.gnome.desktop.interface", "cursor-size"])
        .output()
    {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(s) = val.parse::<u32>() {
                info.cursor_size = Some(s);
            }
        }
    }

    if let Ok(output) = crate::modules::system_command("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_ascii_lowercase();
            if val.contains("dark") {
                info.dark_mode = true;
            }
        }
    }

    info
}

#[cfg(not(windows))]
fn get_config_dir() -> PathBuf {
    if let Ok(cfg) = std::env::var("XDG_CONFIG_HOME") {
        if !cfg.is_empty() {
            return PathBuf::from(cfg);
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        return Path::new(&home).join(".config");
    }

    PathBuf::from("/root/.config")
}

/// Parses Windows light/dark theme preference from AppsUseLightTheme DWORD.
pub fn parse_windows_theme(apps_use_light_theme: u32) -> ThemeInfo {
    let (theme_name, is_dark) = if apps_use_light_theme == 0 {
        ("Dark", true)
    } else {
        ("Light", false)
    };

    ThemeInfo {
        theme: Some(theme_name.to_string()),
        dark_mode: is_dark,
        source: Some(ThemeSource::Windows),
        ..Default::default()
    }
}

/// Detects theme information across GTK 3/4, KDE Plasma, XFCE, and GSettings.
/// Probing precedence:
/// 1. GTK 3.0 / 4.0 `settings.ini` (standard for GNOME/Cinnamon/MATE/modern apps)
/// 2. KDE Plasma `kdeglobals` (Qt/KDE color schemes and look-and-feel packages)
/// 3. XFCE `xsettings.xml` (xfconf channel storage)
/// 4. Legacy GTK 2 `~/.gtkrc-2.0`
/// 5. GSettings dconf query for active GNOME desktop interface schemas
/// 6. `$GTK_THEME` environment variable override
#[cfg(not(windows))]
static THEME_CACHE: std::sync::OnceLock<Option<ThemeInfo>> = std::sync::OnceLock::new();

#[cfg(not(windows))]
pub fn detect_theme_info() -> Option<ThemeInfo> {
    THEME_CACHE.get_or_init(detect_theme_info_uncached).clone()
}

#[cfg(not(windows))]
fn detect_theme_info_uncached() -> Option<ThemeInfo> {
    // 0. Fast-path: Check persistent cache
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));

    let cache_file = cache_dir.as_ref().map(|d| d.join("theme_v2.cache"));

    if let Some(ref path) = cache_file {
        if let Ok(content) = std::fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() >= 6 {
                let theme = if lines[0].is_empty() {
                    None
                } else {
                    Some(lines[0].to_string())
                };
                let icon_theme = if lines[1].is_empty() {
                    None
                } else {
                    Some(lines[1].to_string())
                };
                let font = if lines[2].is_empty() {
                    None
                } else {
                    Some(lines[2].to_string())
                };
                let cursor = if lines[3].is_empty() {
                    None
                } else {
                    Some(lines[3].to_string())
                };
                let cursor_size = lines.get(4).and_then(|s| s.parse::<u32>().ok());
                let dark_mode = lines
                    .get(5)
                    .map(|&d| d == "1" || d.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                let source = match lines.get(6).copied().unwrap_or("") {
                    "Gtk" => Some(ThemeSource::Gtk),
                    "Kde" => Some(ThemeSource::Kde),
                    "Xfce" => Some(ThemeSource::Xfce),
                    "GSettings" => Some(ThemeSource::GSettings),
                    "Env" => Some(ThemeSource::Env),
                    _ => None,
                };

                return Some(ThemeInfo {
                    theme,
                    icon_theme,
                    font,
                    cursor,
                    cursor_size,
                    dark_mode,
                    source,
                });
            }
        }
    }

    let config_dir = get_config_dir();
    let mut resolved: Option<ThemeInfo> = None;

    // 1. GTK 3.0 / 4.0 settings.ini
    for gtk_ver in ["gtk-4.0", "gtk-3.0"] {
        let gtk_settings = config_dir.join(gtk_ver).join("settings.ini");
        if gtk_settings.is_file() {
            if let Ok(content) = std::fs::read_to_string(&gtk_settings) {
                let info = parse_gtk_settings_ini(&content);
                if info.theme.is_some() || info.icon_theme.is_some() {
                    resolved = Some(info);
                    break;
                }
            }
        }
    }

    // 2. KDE Plasma kdeglobals
    if resolved.is_none() {
        let kde_globals = config_dir.join("kdeglobals");
        if kde_globals.is_file() {
            if let Ok(content) = std::fs::read_to_string(&kde_globals) {
                let info = parse_kde_globals(&content);
                if info.theme.is_some() || info.icon_theme.is_some() {
                    resolved = Some(info);
                }
            }
        }
    }

    // 3. XFCE xsettings.xml
    if resolved.is_none() {
        let xfce_settings = config_dir.join("xfce4/xfconf/xfce-perchannel-xml/xsettings.xml");
        if xfce_settings.is_file() {
            if let Ok(content) = std::fs::read_to_string(&xfce_settings) {
                let info = parse_xfce_xsettings(&content);
                if info.theme.is_some() || info.icon_theme.is_some() {
                    resolved = Some(info);
                }
            }
        }
    }

    // 4. GTK 2 ~/.gtkrc-2.0
    if resolved.is_none() {
        if let Ok(home) = std::env::var("HOME") {
            let gtk2_rc = Path::new(&home).join(".gtkrc-2.0");
            if gtk2_rc.is_file() {
                if let Ok(content) = std::fs::read_to_string(&gtk2_rc) {
                    let info = parse_gtk_settings_ini(&content);
                    if info.theme.is_some() || info.icon_theme.is_some() {
                        resolved = Some(info);
                    }
                }
            }
        }
    }

    // 5. GSettings fallback for GNOME sessions
    if resolved.is_none() {
        let gsettings_info = query_gsettings_theme();
        if gsettings_info.theme.is_some() || gsettings_info.icon_theme.is_some() {
            resolved = Some(gsettings_info);
        }
    }

    // 6. Environment variable fallbacks
    if resolved.is_none() {
        if let Ok(theme) = std::env::var("GTK_THEME") {
            if !theme.is_empty() {
                resolved = Some(ThemeInfo {
                    theme: Some(theme),
                    source: Some(ThemeSource::Env),
                    ..Default::default()
                });
            }
        }
    }

    // Save to persistent cache
    if let Some(ref info) = resolved {
        if let Some(ref dir) = cache_dir {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Some(ref path) = cache_file {
            let src_str = match info.source {
                Some(ThemeSource::Gtk) => "Gtk",
                Some(ThemeSource::Kde) => "Kde",
                Some(ThemeSource::Xfce) => "Xfce",
                Some(ThemeSource::GSettings) => "GSettings",
                Some(ThemeSource::Env) => "Env",
                Some(ThemeSource::Windows) => "Windows",
                None => "",
            };
            let serialized = format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
                info.theme.as_deref().unwrap_or(""),
                info.icon_theme.as_deref().unwrap_or(""),
                info.font.as_deref().unwrap_or(""),
                info.cursor.as_deref().unwrap_or(""),
                info.cursor_size
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or(""),
                if info.dark_mode { "1" } else { "0" },
                src_str
            );
            let _ = std::fs::write(path, serialized);
        }
    }

    resolved
}

/// Detects active Windows application theme (Dark / Light) from user registry.
#[cfg(windows)]
pub fn detect_theme_info() -> Option<ThemeInfo> {
    use crate::modules::win_util::ffi;
    let key = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
    let light_theme =
        ffi::reg_read_u32(ffi::HKEY_CURRENT_USER, key, "AppsUseLightTheme").unwrap_or(1);
    Some(parse_windows_theme(light_theme))
}

/// Formats the theme output string with optional framework tag.
pub fn format_theme_value(info: &ThemeInfo) -> Option<String> {
    info.theme.as_ref().map(|theme| {
        let suffix = match info.source {
            Some(source) => format!(" {}", source.label_suffix()),
            None => String::new(),
        };

        if info.dark_mode && !theme.to_ascii_lowercase().contains("dark") {
            format!("{} (dark){}", theme, suffix)
        } else {
            format!("{}{}", theme, suffix)
        }
    })
}

/// Formats the icon theme output string.
pub fn format_icons_value(info: &ThemeInfo) -> Option<String> {
    info.icon_theme.as_ref().map(|icons| match info.source {
        Some(source) => format!("{} {}", icons, source.label_suffix()),
        None => icons.clone(),
    })
}

pub struct ThemeCollector;

impl Collector for ThemeCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Theme
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let info = detect_theme_info()?;
        let value = format_theme_value(&info)?;

        Some(ModuleOutput {
            id: ModuleId::Theme,
            label: "Theme".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

pub struct IconsCollector;

impl Collector for IconsCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Icons
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let info = detect_theme_info()?;
        let value = format_icons_value(&info)?;

        Some(ModuleOutput {
            id: ModuleId::Icons,
            label: "Icons".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

/// Formats the system desktop interface font string.
pub fn format_font_value(info: &ThemeInfo) -> Option<String> {
    info.font.as_ref().map(|font| match info.source {
        Some(source) => format!("{} {}", font, source.label_suffix()),
        None => font.clone(),
    })
}

pub struct FontCollector;

impl Collector for FontCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Font
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let info = detect_theme_info()?;
        let value = format_font_value(&info)?;

        Some(ModuleOutput {
            id: ModuleId::Font,
            label: "Font".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

/// Formats the desktop cursor theme and size string.
pub fn format_cursor_value(info: &ThemeInfo) -> Option<String> {
    let cursor = info.cursor.as_ref()?;
    let with_size = match info.cursor_size {
        Some(size) => format!("{} ({}px)", cursor, size),
        None => cursor.clone(),
    };
    Some(match info.source {
        Some(source) => format!("{} {}", with_size, source.label_suffix()),
        None => with_size,
    })
}

pub struct CursorCollector;

impl Collector for CursorCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Cursor
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let info = detect_theme_info()?;
        let value = format_cursor_value(&info)?;

        Some(ModuleOutput {
            id: ModuleId::Cursor,
            label: "Cursor".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_gtk_settings_ini_standard() {
        let content = r#"
[Settings]
gtk-theme-name=Adwaita-dark
gtk-icon-theme-name=Papirus-Dark
gtk-font-name=Cantarell 11
gtk-cursor-theme-name=Adwaita
gtk-application-prefer-dark-theme=1
"#;
        let info = parse_gtk_settings_ini(content);
        assert_eq!(info.theme.as_deref(), Some("Adwaita-dark"));
        assert_eq!(info.icon_theme.as_deref(), Some("Papirus-Dark"));
        assert_eq!(info.font.as_deref(), Some("Cantarell 11"));
        assert_eq!(info.cursor.as_deref(), Some("Adwaita"));
        assert!(info.dark_mode);

        let formatted = format_theme_value(&info);
        assert_eq!(formatted.as_deref(), Some("Adwaita-dark [GTK]"));
    }

    #[test]
    fn test_parse_gtk_settings_ini_quotes_and_spaces() {
        let content = r#"
[Settings]
gtk-theme-name = "Catppuccin-Mocha-Standard-Blue-Dark"
gtk-icon-theme-name = 'Papirus'
gtk-application-prefer-dark-theme = true
"#;
        let info = parse_gtk_settings_ini(content);
        assert_eq!(
            info.theme.as_deref(),
            Some("Catppuccin-Mocha-Standard-Blue-Dark")
        );
        assert_eq!(info.icon_theme.as_deref(), Some("Papirus"));
        assert!(info.dark_mode);
    }

    #[test]
    fn test_parse_kde_globals_standard() {
        let content = r#"
[KDE]
LookAndFeelPackage=org.kde.breezedark.desktop
widgetStyle=Breeze

[Icons]
Theme=breeze-dark

[General]
ColorScheme=BreezeDark
"#;
        let info = parse_kde_globals(content);
        assert_eq!(info.theme.as_deref(), Some("breeze-Dark"));
        assert_eq!(info.icon_theme.as_deref(), Some("breeze-dark"));
        assert!(info.dark_mode);
        assert_eq!(info.source, Some(ThemeSource::Kde));

        let formatted = format_theme_value(&info);
        assert_eq!(formatted.as_deref(), Some("breeze-Dark [Qt/KDE]"));
    }

    #[test]
    fn test_parse_xfce_xsettings() {
        let content = r#"
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xsettings" version="1.0">
  <property name="Net" type="empty">
    <property name="ThemeName" type="string" value="Greybird"/>
    <property name="IconThemeName" type="string" value="elementary-xfce"/>
  </property>
</channel>
"#;
        let info = parse_xfce_xsettings(content);
        assert_eq!(info.theme.as_deref(), Some("Greybird"));
        assert_eq!(info.icon_theme.as_deref(), Some("elementary-xfce"));
        assert_eq!(info.source, Some(ThemeSource::Xfce));

        let formatted = format_theme_value(&info);
        assert_eq!(formatted.as_deref(), Some("Greybird [XFCE]"));
    }

    #[test]
    fn test_format_theme_dark_mode_annotation() {
        let info = ThemeInfo {
            theme: Some("Adwaita".to_string()),
            dark_mode: true,
            source: Some(ThemeSource::GSettings),
            ..Default::default()
        };

        let formatted = format_theme_value(&info);
        assert_eq!(formatted.as_deref(), Some("Adwaita (dark) [GTK/GNOME]"));
    }

    #[test]
    fn test_parse_windows_theme() {
        // Dark theme (0)
        let dark = parse_windows_theme(0);
        assert_eq!(dark.theme.as_deref(), Some("Dark"));
        assert!(dark.dark_mode);
        assert_eq!(dark.source, Some(ThemeSource::Windows));
        assert_eq!(format_theme_value(&dark).as_deref(), Some("Dark [Windows]"));

        // Light theme (1)
        let light = parse_windows_theme(1);
        assert_eq!(light.theme.as_deref(), Some("Light"));
        assert!(!light.dark_mode);
        assert_eq!(light.source, Some(ThemeSource::Windows));
        assert_eq!(
            format_theme_value(&light).as_deref(),
            Some("Light [Windows]")
        );
    }

    #[test]
    fn test_format_cursor_value_with_and_without_size() {
        let info_with_size = ThemeInfo {
            cursor: Some("Adwaita".to_string()),
            cursor_size: Some(24),
            source: Some(ThemeSource::Gtk),
            ..Default::default()
        };
        assert_eq!(
            format_cursor_value(&info_with_size).as_deref(),
            Some("Adwaita (24px) [GTK]")
        );

        let info_no_size = ThemeInfo {
            cursor: Some("Breeze_Snow".to_string()),
            cursor_size: None,
            source: Some(ThemeSource::Kde),
            ..Default::default()
        };
        assert_eq!(
            format_cursor_value(&info_no_size).as_deref(),
            Some("Breeze_Snow [Qt/KDE]")
        );
    }

    #[test]
    fn test_parse_gtk_settings_ini_cursor_size() {
        let content = r#"
[Settings]
gtk-cursor-theme-name=Bibata-Modern-Classic
gtk-cursor-theme-size=28
"#;
        let info = parse_gtk_settings_ini(content);
        assert_eq!(info.cursor.as_deref(), Some("Bibata-Modern-Classic"));
        assert_eq!(info.cursor_size, Some(28));
        assert_eq!(
            format_cursor_value(&info).as_deref(),
            Some("Bibata-Modern-Classic (28px) [GTK]")
        );
    }
}
