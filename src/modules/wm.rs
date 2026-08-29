use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;

#[cfg(not(windows))]
const KNOWN_WMS: &[&str] = &[
    "kwin_wayland",
    "kwin_x11",
    "kwin",
    "mutter",
    "gnome-shell",
    "xfwm4",
    "muffin",
    "marco",
    "sway",
    "hyprland",
    "i3",
    "bspwm",
    "dwm",
    "awesome",
    "xmonad",
    "qtile",
    "openbox",
    "fluxbox",
    "enlightenment",
    "compiz",
    "weston",
    "wayfire",
    "river",
];

/// Parses the WSLg version from the contents of `/mnt/wslg/versions.txt`.
pub fn parse_wslg_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("WSLg") {
            if let Some((_, ver)) = trimmed.split_once(':') {
                let clean = ver.trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

/// Probes the active WSLg version from `/mnt/wslg/versions.txt`.
#[cfg(not(windows))]
pub fn detect_wslg_version() -> Option<String> {
    fs::read_to_string("/mnt/wslg/versions.txt")
        .ok()
        .and_then(|c| parse_wslg_version(&c))
}

/// Probes active Window Manager from running processes, environment, or WSLg.
#[cfg(not(windows))]
pub fn detect_wm() -> Option<String> {
    // 1. WSLg environment check: Weston Wayland server provides X11/Wayland bridge on /mnt/wslg
    if (fs::metadata("/mnt/wslg").is_ok() || std::env::var_os("WSL_DISTRO_NAME").is_some())
        && (std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some())
    {
        if let Some(wslg_ver) = detect_wslg_version() {
            return Some(format!("WSLg {} (Wayland)", wslg_ver));
        } else {
            return Some("WSLg (Wayland)".to_string());
        }
    }

    // 2. Fast path: Direct $XDG_CURRENT_DESKTOP / compositor socket mapping (<0.01ms)
    if let Ok(cur_de) = std::env::var("XDG_CURRENT_DESKTOP") {
        let de_lower = cur_de.to_lowercase();
        if de_lower.contains("gnome") {
            return Some("Mutter".to_string());
        } else if de_lower.contains("kde") || de_lower.contains("plasma") {
            return Some("KWin".to_string());
        } else if de_lower.contains("xfce") {
            return Some("Xfwm4".to_string());
        } else if de_lower.contains("cinnamon") {
            return Some("Muffin".to_string());
        } else if de_lower.contains("mate") {
            return Some("Marco".to_string());
        }
    }

    // 3. Scan `/proc` PID directories for active known WM process binaries
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                if file_name
                    .to_string_lossy()
                    .chars()
                    .all(|c| c.is_ascii_digit())
                {
                    if let Ok(comm) = fs::read_to_string(path.join("comm")) {
                        let comm_clean = comm.trim().to_lowercase();
                        for &wm in KNOWN_WMS {
                            if comm_clean == wm {
                                let display_name = match wm {
                                    "kwin_wayland" | "kwin_x11" | "kwin" => "KWin",
                                    "mutter" | "gnome-shell" => "Mutter",
                                    "xfwm4" => "Xfwm4",
                                    "muffin" => "Muffin",
                                    "marco" => "Marco",
                                    "sway" => "Sway",
                                    "hyprland" => "Hyprland",
                                    "i3" => "i3",
                                    "bspwm" => "bspwm",
                                    "dwm" => "dwm",
                                    "awesome" => "awesome",
                                    "xmonad" => "xmonad",
                                    "qtile" => "qtile",
                                    "openbox" => "Openbox",
                                    "fluxbox" => "Fluxbox",
                                    "enlightenment" => "Enlightenment",
                                    "compiz" => "Compiz",
                                    "weston" => "Weston",
                                    "wayfire" => "Wayfire",
                                    "river" => "River",
                                    _ => wm,
                                };
                                return Some(display_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback to desktop session environment hints if /proc is masked or restricted
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        let de_lower = de.to_lowercase();
        if de_lower.contains("gnome") {
            return Some("Mutter".to_string());
        } else if de_lower.contains("kde") {
            return Some("KWin".to_string());
        } else if de_lower.contains("xfce") {
            return Some("Xfwm4".to_string());
        } else if de_lower.contains("cinnamon") {
            return Some("Muffin".to_string());
        } else if de_lower.contains("mate") {
            return Some("Marco".to_string());
        } else if de_lower.contains("sway") {
            return Some("Sway".to_string());
        } else if de_lower.contains("hyprland") {
            return Some("Hyprland".to_string());
        } else if de_lower.contains("i3") {
            return Some("i3".to_string());
        }
    }

    None
}

/// Returns Desktop Window Manager on Windows.
#[cfg(windows)]
pub fn detect_wm() -> Option<String> {
    Some("Desktop Window Manager (DWM)".to_string())
}

pub struct WmCollector;

impl Collector for WmCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Wm
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let wm = detect_wm()?;
        Some(ModuleOutput {
            id: ModuleId::Wm,
            label: "WM".to_string(),
            value: wm,
            custom_rendered: None,
        })
    }
}

/// Probes active Window Manager decoration/window theme.
#[cfg(not(windows))]
pub fn detect_wm_theme() -> Option<String> {
    let wm = detect_wm()?;
    let wm_lower = wm.to_lowercase();

    if wm_lower.contains("mutter") || wm_lower.contains("gnome") {
        return Some("Adwaita".to_string());
    }

    if wm_lower.contains("kwin") {
        let home = std::env::var("HOME").ok()?;
        let kwinrc_path = std::path::Path::new(&home).join(".config/kwinrc");
        if let Ok(content) = fs::read_to_string(kwinrc_path) {
            for line in content.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim().eq_ignore_ascii_case("theme")
                        || k.trim().eq_ignore_ascii_case("PluginName")
                    {
                        let clean = v.trim();
                        if !clean.is_empty() {
                            return Some(clean.to_string());
                        }
                    }
                }
            }
        }
        return Some("Breeze".to_string());
    }

    if wm_lower.contains("xfwm") {
        return Some("Default".to_string());
    }

    None
}

#[cfg(windows)]
pub fn detect_wm_theme() -> Option<String> {
    Some("Mica / DWM".to_string())
}

pub struct WmThemeCollector;

impl Collector for WmThemeCollector {
    fn id(&self) -> ModuleId {
        ModuleId::WmTheme
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let theme = detect_wm_theme()?;
        Some(ModuleOutput {
            id: ModuleId::WmTheme,
            label: "WM Theme".to_string(),
            value: theme,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_wm_live() {
        let _ = detect_wm();
    }

    #[test]
    fn test_parse_wslg_version() {
        let sample = "WSLg ( x86_64 ): 1.0.73.2\nBuilt at: Mon May 18 22:31:53 UTC 2026\n";
        assert_eq!(parse_wslg_version(sample), Some("1.0.73.2".to_string()));

        let sample_arm = "WSLg ( aarch64 ): 1.0.65.0\n";
        assert_eq!(parse_wslg_version(sample_arm), Some("1.0.65.0".to_string()));

        let empty = "Azure Linux: VERSION=\"3.0\"\n";
        assert_eq!(parse_wslg_version(empty), None);
    }
}
