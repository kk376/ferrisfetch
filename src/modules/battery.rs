use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(unix)]
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryInfo {
    pub capacity: u8,
    pub status: String,
}

/// Parses Windows SYSTEM_POWER_STATUS fields into BatteryInfo.
pub fn parse_windows_battery_status(
    ac_line_status: u8,
    battery_flag: u8,
    battery_life_percent: u8,
) -> Option<BatteryInfo> {
    // BatteryFlag: 128 indicates no system battery, 255 indicates unknown status
    if battery_flag == 128 || battery_life_percent > 100 {
        return None;
    }

    let is_charging = (battery_flag & 8) != 0;
    let is_ac = ac_line_status == 1;

    let status = if is_charging {
        "Charging".to_string()
    } else if is_ac {
        if battery_life_percent >= 99 {
            "Full [AC]".to_string()
        } else {
            "AC Connected".to_string()
        }
    } else {
        "Discharging".to_string()
    };

    Some(BatteryInfo {
        capacity: battery_life_percent,
        status,
    })
}

/// Checks if any AC adapter power supply (`AC`, `ACAD`, `Mains`) is connected.
#[cfg(not(windows))]
fn is_ac_online() -> bool {
    let power_supply_dir = "/sys/class/power_supply";
    if let Ok(entries) = fs::read_dir(power_supply_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.starts_with("ac")
                || name.starts_with("mains")
                || name.starts_with("acad")
                || name.starts_with("adp")
            {
                if let Ok(online) = fs::read_to_string(entry.path().join("online")) {
                    if online.trim() == "1" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Probes battery capacity and state from `/sys/class/power_supply/BAT*`.
#[cfg(not(windows))]
pub fn detect_battery() -> Option<BatteryInfo> {
    let power_supply_dir = "/sys/class/power_supply";
    let entries = fs::read_dir(power_supply_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name()?.to_string_lossy().to_lowercase();

        if !file_name.starts_with("bat") && !file_name.starts_with("battery") {
            continue;
        }

        // Read percentage capacity (0-100)
        let capacity_str = fs::read_to_string(path.join("capacity")).ok()?;
        let capacity = capacity_str.trim().parse::<u8>().ok()?;

        // Read status (Charging, Discharging, Full, Not charging)
        let raw_status = fs::read_to_string(path.join("status"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let ac_online = is_ac_online();

        // When battery threshold limits (e.g. 80%) are enabled in BIOS, status reports "Not charging" while connected to AC
        let status = if raw_status.eq_ignore_ascii_case("not charging") {
            if ac_online {
                "AC Connected".to_string()
            } else {
                "Not charging".to_string()
            }
        } else if raw_status.eq_ignore_ascii_case("charging") {
            "Charging".to_string()
        } else if raw_status.eq_ignore_ascii_case("discharging") {
            "Discharging".to_string()
        } else if raw_status.eq_ignore_ascii_case("full") {
            if ac_online {
                "Full [AC]".to_string()
            } else {
                "Full".to_string()
            }
        } else {
            raw_status
        };

        return Some(BatteryInfo { capacity, status });
    }

    None
}

/// Probes battery status on Windows via Win32 GetSystemPowerStatus API.
#[cfg(windows)]
pub fn detect_battery() -> Option<BatteryInfo> {
    use crate::modules::win_util::ffi;
    unsafe {
        let mut status = std::mem::zeroed::<ffi::SYSTEM_POWER_STATUS>();
        if ffi::GetSystemPowerStatus(&mut status) != 0 {
            return parse_windows_battery_status(
                status.ACLineStatus,
                status.BatteryFlag,
                status.BatteryLifePercent,
            );
        }
    }
    None
}

pub struct BatteryCollector;

impl Collector for BatteryCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Battery
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let info = detect_battery()?;
        let value = format!("{}% [{}]", info.capacity, info.status);
        Some(ModuleOutput {
            id: ModuleId::Battery,
            label: "Battery".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_parsing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bat_dir = temp_dir.path().join("BAT1");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(
            bat_dir.join("model_name"),
            "Microsoft Hyper-V Virtual Battery\n",
        )
        .unwrap();
        fs::write(bat_dir.join("capacity"), "97\n").unwrap();
        fs::write(bat_dir.join("status"), "Not charging\n").unwrap();

        let cap = fs::read_to_string(bat_dir.join("capacity"))
            .unwrap()
            .trim()
            .parse::<u8>()
            .unwrap();
        assert_eq!(cap, 97);
    }

    #[test]
    fn test_parse_windows_battery_status() {
        // Desktop PC (no battery)
        assert_eq!(parse_windows_battery_status(1, 128, 255), None);

        // Laptop plugged in and charging at 65%
        let charging = parse_windows_battery_status(1, 8, 65).unwrap();
        assert_eq!(charging.capacity, 65);
        assert_eq!(charging.status, "Charging");

        // Laptop discharging on battery at 80%
        let discharging = parse_windows_battery_status(0, 0, 80).unwrap();
        assert_eq!(discharging.capacity, 80);
        assert_eq!(discharging.status, "Discharging");

        // Laptop full on AC
        let full = parse_windows_battery_status(1, 0, 100).unwrap();
        assert_eq!(full.capacity, 100);
        assert_eq!(full.status, "Full [AC]");
    }
}
