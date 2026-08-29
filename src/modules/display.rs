use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayInfo {
    pub name: Option<String>,
    pub resolution: String,
    pub refresh_rate: Option<u32>,
    pub size_inches: Option<u32>,
    pub display_type: Option<String>,
}

/// Parses raw 128-byte EDID binary block into structured DisplayInfo.
pub fn parse_edid_binary(data: &[u8], connector_name: &str) -> Option<DisplayInfo> {
    if data.len() < 128 {
        return None;
    }
    // Verify standard EDID header magic (00 FF FF FF FF FF FF 00)
    if data[0..8] != [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00] {
        return None;
    }

    // 1. Manufacturer Code (PNP ID)
    let mfg_id_raw = ((data[8] as u16) << 8) | (data[9] as u16);
    let c1 = (((mfg_id_raw >> 10) & 0x1F) as u8 + b'A' - 1) as char;
    let c2 = (((mfg_id_raw >> 5) & 0x1F) as u8 + b'A' - 1) as char;
    let c3 = ((mfg_id_raw & 0x1F) as u8 + b'A' - 1) as char;
    let prod_code = (data[10] as u16) | ((data[11] as u16) << 8);
    let mfg_code = format!("{}{}{}{:04X}", c1, c2, c3, prod_code);

    let name = Some(mfg_code);

    // 3. Physical screen size in inches from byte 21 (w_cm) and byte 22 (h_cm)
    let w_cm = data[21] as f64;
    let h_cm = data[22] as f64;
    let size_inches = if w_cm > 0.0 && h_cm > 0.0 {
        let diag_cm = (w_cm * w_cm + h_cm * h_cm).sqrt();
        let inches = (diag_cm / 2.54).round() as u32;
        if inches > 0 {
            Some(inches)
        } else {
            None
        }
    } else {
        None
    };

    // 4. Detailed timing 1 resolution & refresh rate
    let pixel_clock_10khz = (data[54] as u32) | ((data[55] as u32) << 8);
    let h_active = (data[56] as u32) | (((data[58] >> 4) as u32) << 8);
    let h_blank = (data[57] as u32) | (((data[58] & 0x0F) as u32) << 8);
    let v_active = (data[59] as u32) | (((data[61] >> 4) as u32) << 8);
    let v_blank = (data[60] as u32) | (((data[61] & 0x0F) as u32) << 8);

    let h_total = h_active + h_blank;
    let v_total = v_active + v_blank;

    let (res, hz) = if pixel_clock_10khz > 0 && h_total > 0 && v_total > 0 {
        let refresh = ((pixel_clock_10khz as f64 * 10000.0) / (h_total as f64 * v_total as f64))
            .round() as u32;
        (format!("{}x{}", h_active, v_active), Some(refresh))
    } else {
        ("1920x1080".to_string(), None)
    };

    // 5. Display type [Built-in] vs [External]
    let conn_lower = connector_name.to_lowercase();
    let display_type = if conn_lower.contains("edp")
        || conn_lower.contains("lvds")
        || conn_lower.contains("dsi")
    {
        Some("[Built-in]".to_string())
    } else if conn_lower.contains("hdmi")
        || conn_lower.contains("dp")
        || conn_lower.contains("vga")
        || conn_lower.contains("dvi")
    {
        Some("[External]".to_string())
    } else {
        None
    };

    Some(DisplayInfo {
        name,
        resolution: res,
        refresh_rate: hz,
        size_inches,
        display_type,
    })
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
                        name: None,
                        resolution: res.to_string(),
                        refresh_rate: rate,
                        size_inches: None,
                        display_type: None,
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
                        name: None,
                        resolution: res,
                        refresh_rate: hz,
                        size_inches: None,
                        display_type: None,
                    });
                }
            }
        }
    }
    None
}

/// Probes display resolution, refresh rate, size, and monitor name from sysfs DRM or display servers.
pub fn detect_display() -> Option<DisplayInfo> {
    // 1. Sysfs DRM modes + EDID fast path (<0.05ms) for Linux (Wayland, X11, KMS, and TTY consoles)
    let drm_dir = "/sys/class/drm";
    if let Ok(entries) = fs::read_dir(drm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(status) = fs::read_to_string(path.join("status")) {
                if status.trim() == "connected" {
                    let conn_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if let Ok(edid_bytes) = fs::read(path.join("edid")) {
                        if let Some(mut info) = parse_edid_binary(&edid_bytes, &conn_name) {
                            if let Ok(modes) = fs::read_to_string(path.join("modes")) {
                                if let Some(first_mode) = modes.lines().next() {
                                    let clean = first_mode.trim();
                                    if clean.contains('x') {
                                        info.resolution = clean.to_string();
                                    }
                                }
                            }
                            return Some(info);
                        }
                    }

                    if let Ok(modes) = fs::read_to_string(path.join("modes")) {
                        if let Some(first_mode) = modes.lines().next() {
                            let clean = first_mode.trim();
                            if clean.contains('x') {
                                return Some(DisplayInfo {
                                    name: None,
                                    resolution: clean.to_string(),
                                    refresh_rate: None,
                                    size_inches: None,
                                    display_type: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Fallback: Query xrandr or wlr-randr if graphical display session is active and DRM sysfs is unavailable
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(output) = Command::new("xrandr").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(info) = parse_xrandr_output(&text) {
                    return Some(info);
                }
            }
        }

        if let Ok(output) = Command::new("wlr-randr").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(info) = parse_wlr_randr_output(&text) {
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

        let label = match info.name {
            Some(ref n) => format!("Display ({})", n),
            None => "Display".to_string(),
        };

        let mut main_str = info.resolution;
        if let Some(inch) = info.size_inches {
            main_str.push_str(&format!(" in {}\"", inch));
        }
        let mut sub_parts = vec![main_str];
        if let Some(hz) = info.refresh_rate {
            sub_parts.push(format!("{} Hz", hz));
        }
        let mut value = sub_parts.join(", ");
        if let Some(ref dtype) = info.display_type {
            value.push(' ');
            value.push_str(dtype);
        }

        Some(ModuleOutput {
            id: ModuleId::Display,
            label,
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

    #[test]
    fn test_parse_edid_binary_builtin() {
        let mut edid = [0u8; 128];
        // Standard header
        edid[0..8].copy_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
        // AUO manufacturer code (0x06, 0xAF)
        edid[8] = 0x06;
        edid[9] = 0xAF;
        // Product code: 0xD0A2
        edid[10] = 0xA2;
        edid[11] = 0xD0;
        // Physical size: 34cm x 19cm (15 inch diagonal)
        edid[21] = 34;
        edid[22] = 19;

        // Pixel clock: 33742 (337.42 MHz = 0x83CE)
        edid[54] = 0xCE;
        edid[55] = 0x83;
        // H active: 1920 (0x780), H blank: 160 (0x0A0) -> H total: 2080
        edid[56] = 0x80;
        edid[57] = 0xA0;
        edid[58] = 0x70;
        // V active: 1080 (0x438), V blank: 45 (0x02D) -> V total: 1125
        edid[59] = 0x38;
        edid[60] = 0x2D;
        edid[61] = 0x40;

        let info = parse_edid_binary(&edid, "card1-eDP-1").unwrap();
        assert_eq!(info.name.as_deref(), Some("AUOD0A2"));
        assert_eq!(info.resolution, "1920x1080");
        assert_eq!(info.size_inches, Some(15));
        assert_eq!(info.refresh_rate, Some(144));
        assert_eq!(info.display_type.as_deref(), Some("[Built-in]"));
    }
}
