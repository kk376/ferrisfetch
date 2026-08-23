use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;

#[cfg(not(windows))]
const KNOWN_WMS: &[&str] = &[
    "i3", "bspwm", "awesome", "dwm", "openbox", "xmonad", "qtile", "mutter", "kwin", "xfwm4",
    "compiz", "marco", "sway", "hyprland", "wayfire", "river", "labwc",
];

/// Identifies active Wayland window manager from environment signatures.
pub fn detect_wayland_wm_from_env(
    hyprland_sig: bool,
    swaysock: bool,
    wayfire_cfg: bool,
    river_sock: bool,
    labwc_pid: bool,
) -> Option<&'static str> {
    if hyprland_sig {
        Some("Hyprland")
    } else if swaysock {
        Some("Sway")
    } else if wayfire_cfg {
        Some("Wayfire")
    } else if river_sock {
        Some("River")
    } else if labwc_pid {
        Some("labwc")
    } else {
        None
    }
}

/// Formats DE, WM, and session type into a clean string.
pub fn format_desktop_info(
    de: Option<&str>,
    wm: Option<&str>,
    session_type: Option<&str>,
) -> Option<String> {
    let de_clean = de.map(|s| s.trim()).filter(|s| !s.is_empty());
    let wm_clean = wm.map(|s| s.trim()).filter(|s| !s.is_empty());
    let sess_clean = session_type
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.to_lowercase() != "tty");

    match (de_clean, wm_clean, sess_clean) {
        (Some(d), Some(w), Some(sess)) => {
            if d.to_lowercase() == w.to_lowercase()
                || (d.to_lowercase() == "gnome" && w.to_lowercase() == "mutter")
                || (d.to_lowercase() == "kde" && w.to_lowercase() == "kwin")
            {
                Some(format!("{} ({})", d, capitalize_first(sess)))
            } else {
                Some(format!("{} (WM: {}, {})", d, w, capitalize_first(sess)))
            }
        }
        (Some(d), Some(w), None) => {
            if d.to_lowercase() == w.to_lowercase() {
                Some(d.to_string())
            } else {
                Some(format!("{} (WM: {})", d, w))
            }
        }
        (Some(d), None, Some(sess)) => Some(format!("{} ({})", d, capitalize_first(sess))),
        (Some(d), None, None) => Some(d.to_string()),
        (None, Some(w), Some(sess)) => Some(format!("{} ({})", w, capitalize_first(sess))),
        (None, Some(w), None) => Some(w.to_string()),
        (None, None, _) => None,
    }
}

/// Probes desktop environment version from metadata files or fast version queries with persistent caching.
#[cfg(not(windows))]
pub fn detect_de_version(de_name: &str) -> Option<String> {
    let lower = de_name.to_lowercase();

    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));
    let cache_file = cache_dir
        .as_ref()
        .map(|d| d.join(format!("de_{}.cache", lower.replace(' ', "_"))));

    if let Some(ref path) = cache_file {
        if let Ok(cached) = fs::read_to_string(path) {
            let trimmed = cached.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    let detected: Option<String> = if lower.contains("gnome") {
        // Fast path 1: Parse GNOME version XML metadata directly (<0.1ms) avoiding spawning gnome-shell
        if let Ok(xml) = fs::read_to_string("/usr/share/gnome/gnome-version.xml") {
            if let (Some(p_start), Some(m_start)) = (xml.find("<platform>"), xml.find("<minor>")) {
                let platform = xml[p_start + 10..]
                    .split("</platform>")
                    .next()
                    .unwrap_or("")
                    .trim();
                let minor = xml[m_start + 7..]
                    .split("</minor>")
                    .next()
                    .unwrap_or("")
                    .trim();
                let micro = xml
                    .find("<micro>")
                    .and_then(|idx| xml[idx + 7..].split("</micro>").next())
                    .map(|s| s.trim())
                    .unwrap_or("");
                if !platform.is_empty() && !minor.is_empty() {
                    if !micro.is_empty() && micro != "0" {
                        Some(format!("{}.{}.{}", platform, minor, micro))
                    } else {
                        Some(format!("{}.{}", platform, minor))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else if let Ok(out) = std::process::Command::new("gnome-shell")
            .arg("--version")
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                text.split_whitespace().last().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else if lower.contains("kde") || lower.contains("plasma") {
        if let Ok(ver) = std::env::var("KDE_SESSION_VERSION") {
            Some(ver)
        } else if let Ok(out) = std::process::Command::new("plasmashell")
            .arg("--version")
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                text.split_whitespace().last().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else if lower.contains("xfce") {
        if let Ok(out) = std::process::Command::new("xfce4-session")
            .arg("--version")
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                text.lines()
                    .next()
                    .and_then(|l| l.split_whitespace().last())
                    .map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else if lower.contains("mate") {
        if let Ok(out) = std::process::Command::new("mate-session")
            .arg("--version")
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                text.split_whitespace().last().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else if lower.contains("cinnamon") {
        if let Ok(out) = std::process::Command::new("cinnamon")
            .arg("--version")
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                text.split_whitespace().last().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some(ref ver) = detected {
        if let Some(ref dir) = cache_dir {
            let _ = fs::create_dir_all(dir);
        }
        if let Some(ref path) = cache_file {
            let _ = fs::write(path, ver);
        }
    }

    detected
}

/// Probes the system for active Desktop Environment (DE), Window Manager (WM), and session type.
#[cfg(not(windows))]
pub fn detect_desktop() -> Option<String> {
    let mut de = None;
    let mut wm = None;

    // 1. Detect Desktop Environment from standard desktop environment variables
    if let Ok(cur_de) = std::env::var("XDG_CURRENT_DESKTOP") {
        let clean = cur_de.trim();
        if !clean.is_empty() {
            // Handle colon-separated lists like "ubuntu:GNOME" or "pop:GNOME"
            let primary = clean.split(':').next_back().unwrap_or(clean);
            if let Some(ver) = detect_de_version(primary) {
                de = Some(format!("{} {}", primary, ver));
            } else {
                de = Some(primary.to_string());
            }
        }
    } else if let Ok(sess) = std::env::var("DESKTOP_SESSION") {
        let clean = sess.trim();
        if !clean.is_empty() && clean != "default" {
            if let Some(ver) = detect_de_version(clean) {
                de = Some(format!("{} {}", clean, ver));
            } else {
                de = Some(clean.to_string());
            }
        }
    }

    // 2. Detect standalone Wayland Window Managers via compositor-specific socket variables
    if let Some(wayland_wm) = detect_wayland_wm_from_env(
        std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some(),
        std::env::var_os("SWAYSOCK").is_some(),
        std::env::var_os("WAYFIRE_CONFIG_FILE").is_some(),
        std::env::var_os("RIVER_SOCKET").is_some(),
        std::env::var_os("LABWC_PID").is_some(),
    ) {
        wm = Some(wayland_wm.to_string());
    }

    // 3. Fallback WM check from running processes if WM is not yet identified
    if wm.is_none() {
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.chars().all(|c| c.is_ascii_digit()) {
                    let comm_path = entry.path().join("comm");
                    if let Ok(comm) = fs::read_to_string(comm_path) {
                        let clean_comm = comm.trim().to_lowercase();
                        for &known in KNOWN_WMS {
                            if clean_comm == known {
                                wm = Some(capitalize_first(known));
                                break;
                            }
                        }
                        if wm.is_some() {
                            break;
                        }
                    }
                }
            }
        }
    }

    // 4. Session type (Wayland / X11 / TTY)
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();

    format_desktop_info(de.as_deref(), wm.as_deref(), session_type.as_deref())
}

/// Returns Windows Shell on Windows.
#[cfg(windows)]
pub fn detect_desktop() -> Option<String> {
    Some("Windows Explorer".to_string())
}

pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => {
            if s.eq_ignore_ascii_case("x11") {
                "X11".to_string()
            } else if s.eq_ignore_ascii_case("wayland") {
                "Wayland".to_string()
            } else {
                first.to_uppercase().collect::<String>() + chars.as_str()
            }
        }
        None => String::new(),
    }
}

pub struct DesktopCollector;

impl Collector for DesktopCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Desktop
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let de = detect_desktop()?;
        Some(ModuleOutput {
            id: ModuleId::Desktop,
            label: "Desktop".to_string(),
            value: de,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("wayland"), "Wayland");
        assert_eq!(capitalize_first("x11"), "X11");
        assert_eq!(capitalize_first("gnome"), "Gnome");
    }

    #[test]
    fn test_format_desktop_wayland_wms() {
        assert_eq!(
            detect_wayland_wm_from_env(true, false, false, false, false),
            Some("Hyprland")
        );
        assert_eq!(
            detect_wayland_wm_from_env(false, true, false, false, false),
            Some("Sway")
        );
        assert_eq!(
            detect_wayland_wm_from_env(false, false, false, true, false),
            Some("River")
        );
        assert_eq!(
            detect_wayland_wm_from_env(false, false, false, false, false),
            None
        );
    }

    #[test]
    fn test_format_desktop_info_combinations() {
        // Headless
        assert_eq!(format_desktop_info(None, None, None), None);
        assert_eq!(format_desktop_info(None, None, Some("tty")), None);

        // Sway on Wayland
        assert_eq!(
            format_desktop_info(None, Some("Sway"), Some("wayland")),
            Some("Sway (Wayland)".to_string())
        );

        // GNOME on Wayland with mutter
        assert_eq!(
            format_desktop_info(Some("GNOME"), Some("mutter"), Some("wayland")),
            Some("GNOME (Wayland)".to_string())
        );

        // KDE on X11 with kwin
        assert_eq!(
            format_desktop_info(Some("KDE"), Some("kwin"), Some("x11")),
            Some("KDE (X11)".to_string())
        );

        // Custom WM on X11
        assert_eq!(
            format_desktop_info(Some("XFCE"), Some("i3"), Some("x11")),
            Some("XFCE (WM: i3, X11)".to_string())
        );
    }
}
