use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayInfo {
    pub resolution: String,
    pub refresh_rate: Option<u32>,
}

/// Parses xrandr standard output for current resolution and refresh rate.
pub fn parse_xrandr_output(output: &str) -> Option<DisplayInfo> {
    for line in output.lines() {
        if line.contains('*') {
            // e.g. "   1920x1080     59.96*+"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(res) = parts.first() {
                if res.contains('x') {
                    let mut rate: Option<u32> = None;
                    for part in parts.iter().skip(1) {
                        if part.contains('*') {
                            let clean_rate = part.replace(['*', '+'], "");
                            if let Ok(hz) = clean_rate.trim().parse::<f64>() {
                                rate = Some(hz.round() as u32);
                                break;
                            }
                        }
                    }
                    return Some(DisplayInfo {
                        resolution: res.to_string(),
                        refresh_rate: rate,
                    });
                }
            }
        }
    }
    None
}

/// Parses wlr-randr output for current resolution and refresh rate.
pub fn parse_wlr_randr_output(output: &str) -> Option<DisplayInfo> {
    for line in output.lines() {
        if line.contains("current") || line.contains("Hz") {
            // e.g. "  1366x768 px, 60.000000 Hz (current)"
            let trimmed = line.trim();
            if let Some(px_idx) = trimmed.find("px") {
                let res = trimmed[..px_idx].trim().to_string();
                let hz_part = trimmed[px_idx + 2..].trim();
                let hz = hz_part
                    .split_whitespace()
                    .next()
                    .and_then(|h| h.trim_end_matches(',').parse::<f64>().ok())
                    .map(|f| f.round() as u32);
                if res.contains('x') {
                    return Some(DisplayInfo {
                        resolution: res,
                        refresh_rate: hz,
                    });
                }
            }
        }
    }
    None
}

/// Probes display resolution and refresh rate from sysfs DRM or display servers.
pub fn detect_display() -> Option<DisplayInfo> {
    // 1. Fast cache for X11/Wayland/WSLg query (<0.1ms) avoiding server roundtrips
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));
    let cache_file = cache_dir.as_ref().map(|d| d.join("display.cache"));

    if let Some(ref path) = cache_file {
        if let Ok(cached) = fs::read_to_string(path) {
            let trimmed = cached.trim();
            if !trimmed.is_empty() {
                let parts: Vec<&str> = trimmed.split('@').collect();
                let res = parts[0].trim().to_string();
                let hz = parts
                    .get(1)
                    .and_then(|h| h.trim_end_matches("Hz").trim().parse::<u32>().ok());
                return Some(DisplayInfo {
                    resolution: res,
                    refresh_rate: hz,
                });
            }
        }
    }

    // 2. Sysfs DRM modes fast path (<0.05ms) for Linux (Wayland, X11, KMS, and TTY consoles)
    let drm_dir = "/sys/class/drm";
    if let Ok(entries) = fs::read_dir(drm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(status) = fs::read_to_string(path.join("status")) {
                if status.trim() == "connected" {
                    if let Ok(modes) = fs::read_to_string(path.join("modes")) {
                        if let Some(first_mode) = modes.lines().next() {
                            let clean = first_mode.trim();
                            if clean.contains('x') {
                                let info = DisplayInfo {
                                    resolution: clean.to_string(),
                                    refresh_rate: None,
                                };
                                if let Some(ref dir) = cache_dir {
                                    let _ = fs::create_dir_all(dir);
                                }
                                if let Some(ref path) = cache_file {
                                    let _ = fs::write(path, &info.resolution);
                                }
                                return Some(info);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: Query xrandr or wlr-randr if graphical display session is active and DRM sysfs is unavailable
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(output) = Command::new("xrandr").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(info) = parse_xrandr_output(&text) {
                    if let Some(ref dir) = cache_dir {
                        let _ = fs::create_dir_all(dir);
                    }
                    if let Some(ref path) = cache_file {
                        let rate_str = info
                            .refresh_rate
                            .map(|r| format!(" @ {}Hz", r))
                            .unwrap_or_default();
                        let _ = fs::write(path, format!("{}{}", info.resolution, rate_str));
                    }
                    return Some(info);
                }
            }
        }

        if let Ok(output) = Command::new("wlr-randr").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(info) = parse_wlr_randr_output(&text) {
                    if let Some(ref dir) = cache_dir {
                        let _ = fs::create_dir_all(dir);
                    }
                    if let Some(ref path) = cache_file {
                        let rate_str = info
                            .refresh_rate
                            .map(|r| format!(" @ {}Hz", r))
                            .unwrap_or_default();
                        let _ = fs::write(path, format!("{}{}", info.resolution, rate_str));
                    }
                    return Some(info);
                }
            }
        }
    }

    None
}

pub struct DisplayCollector;

impl Collector for DisplayCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Display
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let info = detect_display()?;
        let value = match info.refresh_rate {
            Some(hz) => format!("{} @ {}Hz", info.resolution, hz),
            None => info.resolution,
        };

        Some(ModuleOutput {
            id: ModuleId::Display,
            label: "Display".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xrandr_output_standard() {
        let sample = r#"
Screen 0: minimum 16 x 16, current 1920 x 1080, maximum 32767 x 32767
rdp-0 connected 1920x1080+0+0 (normal left inverted right x axis y axis) 0mm x 0mm
   1920x1080     59.96*+
   1440x1080     59.99  
"#;
        let info = parse_xrandr_output(sample).unwrap();
        assert_eq!(info.resolution, "1920x1080");
        assert_eq!(info.refresh_rate, Some(60));
    }
}
