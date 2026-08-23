use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;
#[cfg(not(windows))]
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsInfo {
    pub display_name: String,
    pub distro_id: String,
    pub distro_like: Vec<String>,
}

/// Parses standard `/etc/os-release` or `/usr/lib/os-release` file contents per the systemd os-release spec.
pub fn parse_os_release(content: &str) -> OsInfo {
    let mut pretty_name = None;
    let mut name = None;
    let mut version = None;
    let mut id = None;
    let mut id_like = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim();

            // Strip surrounding double/single quotes and unescape escaped quotation marks
            let mut val_str = val;
            if val_str.len() >= 2
                && ((val_str.starts_with('"') && val_str.ends_with('"'))
                    || (val_str.starts_with('\'') && val_str.ends_with('\'')))
            {
                val_str = &val_str[1..val_str.len() - 1];
            }
            let unescaped = val_str.replace("\\\"", "\"").replace("\\\\", "\\");
            let clean_val = unescaped.trim();
            if clean_val.is_empty() {
                continue;
            }

            match key {
                "PRETTY_NAME" => pretty_name = Some(clean_val.to_string()),
                "NAME" => name = Some(clean_val.to_string()),
                "VERSION" | "VERSION_ID" => {
                    if version.is_none() {
                        version = Some(clean_val.to_string());
                    }
                }
                "ID" => id = Some(clean_val.to_lowercase()),
                "ID_LIKE" => id_like = Some(clean_val.to_lowercase()),
                _ => {}
            }
        }
    }

    // PRETTY_NAME is preferred; fall back to "NAME VERSION" or "NAME" if PRETTY_NAME is unset
    let display_name = pretty_name
        .or_else(|| match (name, version) {
            (Some(n), Some(v)) => Some(format!("{} {}", n, v)),
            (Some(n), None) => Some(n),
            _ => None,
        })
        .unwrap_or_else(|| "Linux".to_string());

    let distro_id = id.unwrap_or_else(|| "linux".to_string());
    // ID_LIKE contains a space-separated list of parent distributions per the systemd spec
    let distro_like = id_like
        .map(|s| s.split_whitespace().map(|x| x.to_string()).collect())
        .unwrap_or_default();

    OsInfo {
        display_name,
        distro_id,
        distro_like,
    }
}

/// Formats Windows OS display name according to ProductName, DisplayVersion, and BuildNumber.
/// Handles Microsoft's build >= 22000 Windows 11 branding mapping.
pub fn parse_windows_os_info(
    product_name: &str,
    display_version: Option<&str>,
    build_number: Option<&str>,
) -> OsInfo {
    let build_num: Option<u32> = build_number.and_then(|b| b.trim().parse().ok());
    let is_win11 = build_num.map(|b| b >= 22000).unwrap_or(false);

    let mut name = product_name.trim().to_string();
    if is_win11 && name.contains("Windows 10") {
        name = name.replace("Windows 10", "Windows 11");
    }

    let distro_id = if is_win11 || name.contains("Windows 11") {
        "windows11".to_string()
    } else if name.contains("Windows 10") {
        "windows10".to_string()
    } else if name.contains("Windows 7") {
        "windows7".to_string()
    } else {
        "windows".to_string()
    };

    let mut display_name = name;
    if let Some(dv) = display_version {
        let dv_clean = dv.trim();
        if !dv_clean.is_empty() && !display_name.contains(dv_clean) {
            display_name = format!("{} {}", display_name, dv_clean);
        }
    }

    if let Some(b) = build_number {
        let b_clean = b.trim();
        if !b_clean.is_empty() {
            display_name = format!("{} (Build {})", display_name, b_clean);
        }
    }

    OsInfo {
        display_name,
        distro_id,
        distro_like: Vec::new(),
    }
}

/// Detects the operating system using standard and legacy paths.
#[cfg(not(windows))]
pub fn detect_os() -> OsInfo {
    // 1. Primary standard os-release files (/usr/lib fallback handles stateless/immutable systems)
    for path in &["/etc/os-release", "/usr/lib/os-release"] {
        if let Ok(content) = fs::read_to_string(path) {
            let info = parse_os_release(&content);
            if !info.display_name.is_empty() && info.display_name != "Linux" {
                return info;
            }
        }
    }

    // 2. Legacy distribution release files for pre-systemd environments
    if let Ok(deb) = fs::read_to_string("/etc/debian_version") {
        let trimmed = deb.trim();
        if !trimmed.is_empty() {
            return OsInfo {
                display_name: format!("Debian {}", trimmed),
                distro_id: "debian".to_string(),
                distro_like: Vec::new(),
            };
        }
    }

    if let Ok(rh) = fs::read_to_string("/etc/redhat-release") {
        let trimmed = rh.trim();
        if !trimmed.is_empty() {
            return OsInfo {
                display_name: trimmed.to_string(),
                distro_id: "rhel".to_string(),
                distro_like: vec!["fedora".to_string()],
            };
        }
    }

    if Path::new("/etc/arch-release").exists() {
        return OsInfo {
            display_name: "Arch Linux".to_string(),
            distro_id: "arch".to_string(),
            distro_like: Vec::new(),
        };
    }

    if let Ok(gentoo) = fs::read_to_string("/etc/gentoo-release") {
        let trimmed = gentoo.trim();
        if !trimmed.is_empty() {
            return OsInfo {
                display_name: trimmed.to_string(),
                distro_id: "gentoo".to_string(),
                distro_like: Vec::new(),
            };
        }
    }

    if let Ok(alpine) = fs::read_to_string("/etc/alpine-release") {
        let trimmed = alpine.trim();
        if !trimmed.is_empty() {
            return OsInfo {
                display_name: format!("Alpine Linux {}", trimmed),
                distro_id: "alpine".to_string(),
                distro_like: Vec::new(),
            };
        }
    }

    // 3. Fallback to generic kernel sysname
    OsInfo {
        display_name: "Linux".to_string(),
        distro_id: "linux".to_string(),
        distro_like: Vec::new(),
    }
}

/// Detects Windows OS version and build metadata from the system registry.
#[cfg(windows)]
pub fn detect_os() -> OsInfo {
    use crate::modules::win_util::ffi;
    let key = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let product_name = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "ProductName")
        .unwrap_or_else(|| "Windows".to_string());
    let display_version = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "DisplayVersion")
        .or_else(|| ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "ReleaseId"));
    let build_number = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "CurrentBuildNumber")
        .or_else(|| ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "CurrentBuild"));

    parse_windows_os_info(
        &product_name,
        display_version.as_deref(),
        build_number.as_deref(),
    )
}

pub struct OsCollector;

impl Collector for OsCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Os
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        let arch = crate::modules::kernel::get_uname_info()
            .map(|k| k.architecture)
            .unwrap_or_else(|| "x86_64".to_string());
        let value = format!("{} {}", ctx.os_info.display_name, arch);

        Some(ModuleOutput {
            id: ModuleId::Os,
            label: "OS".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

/// Decodes raw process output bytes as UTF-16LE if null-byte padded, or falls back to UTF-8.
pub fn decode_utf16le_or_utf8(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && (bytes[1] == 0 || bytes[0] == 0) {
        let u16_slice: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16_lossy(&u16_slice)
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

/// Parses the WSL version from `wsl.exe --version` command output text.
pub fn parse_wsl_version_output(text: &str) -> Option<String> {
    for line in text.lines() {
        let clean = line.trim();
        if clean.starts_with("WSL version:") || clean.starts_with("WSL version :") {
            if let Some((_, ver)) = clean.split_once(':') {
                let trimmed = ver.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

/// Probes the host WSL version from persistent cache or `wsl.exe --version`.
#[cfg(not(windows))]
pub fn detect_wsl_version() -> Option<String> {
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));

    let cache_file = cache_dir.as_ref().map(|d| d.join("wsl_version.cache"));

    if let Some(ref path) = cache_file {
        if let Ok(cached) = fs::read_to_string(path) {
            let trimmed = cached.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Fast-path 1: Read directly from /mnt/wslg/versions.txt without spawning wsl.exe (<0.01ms)
    if let Ok(versions_txt) = fs::read_to_string("/mnt/wslg/versions.txt") {
        if let Some(ver) = parse_wsl_version_output(&versions_txt) {
            if let Some(ref dir) = cache_dir {
                let _ = fs::create_dir_all(dir);
            }
            if let Some(ref path) = cache_file {
                let _ = fs::write(path, &ver);
            }
            return Some(ver);
        }
    }

    for cmd in &[
        "wsl.exe",
        "/mnt/c/Windows/System32/wsl.exe",
        "/mnt/c/Program Files/WSL/wsl.exe",
    ] {
        if let Ok(output) = std::process::Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let text = decode_utf16le_or_utf8(&output.stdout);
                if let Some(ver) = parse_wsl_version_output(&text) {
                    if let Some(ref dir) = cache_dir {
                        let _ = fs::create_dir_all(dir);
                    }
                    if let Some(ref path) = cache_file {
                        let _ = fs::write(path, &ver);
                    }
                    return Some(ver);
                }
            }
        }
    }

    None
}

/// Detects hardware product/host model from sysfs DMI or Open Firmware devicetree.
#[cfg(not(windows))]
pub fn detect_host() -> Option<String> {
    // Check DMI product name and version (sysfs /sys/devices/virtual/dmi and /sys/class/dmi)
    let product_name = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
        .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_name"))
        .ok()
        .map(|s| s.trim().to_string());
    let product_version = fs::read_to_string("/sys/devices/virtual/dmi/id/product_version")
        .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_version"))
        .ok()
        .map(|s| s.trim().to_string());

    let is_wsl = fs::read_to_string("/proc/version")
        .map(|v| v.contains("microsoft") || v.contains("WSL"))
        .unwrap_or(false)
        || std::env::var_os("WSL_DISTRO_NAME").is_some();

    let wsl_suffix = if is_wsl {
        if let Some(wsl_ver) = detect_wsl_version() {
            format!("- {}", wsl_ver)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    if let Some(ref name) = product_name {
        let name_lower = name.to_lowercase();
        // Ignore unpopulated SMBIOS placeholder values emitted by motherboard vendors
        let is_invalid = name_lower.is_empty()
            || name_lower == "none"
            || name_lower == "default string"
            || name_lower == "system product name"
            || name_lower == "to be filled by o.e.m."
            || name_lower.starts_with("system manufacturer");

        if !is_invalid {
            let mut full = name.clone();
            if let Some(ref ver) = product_version {
                let ver_lower = ver.to_lowercase();
                if !ver_lower.is_empty()
                    && ver_lower != "none"
                    && ver_lower != "default string"
                    && ver_lower != "to be filled by o.e.m."
                    && ver_lower != "1.0"
                    && ver_lower != name_lower
                {
                    full = format!("{} {}", name, ver);
                }
            }
            if is_wsl && !wsl_suffix.is_empty() {
                full = format!("{} {}", full, wsl_suffix);
            }
            return Some(full);
        }
    }

    // Devicetree model for ARM single-board computers (Raspberry Pi, Pine64, Rockchip) lacking SMBIOS
    for dt_path in &[
        "/sys/firmware/devicetree/base/model",
        "/proc/device-tree/model",
    ] {
        if let Ok(model) = fs::read(dt_path) {
            let clean = String::from_utf8_lossy(&model)
                .trim_matches('\0')
                .trim()
                .to_string();
            if !clean.is_empty() {
                return Some(clean);
            }
        }
    }

    // Motherboard board_name fallback for custom desktop rigs
    for board_path in &[
        "/sys/devices/virtual/dmi/id/board_name",
        "/sys/class/dmi/id/board_name",
    ] {
        if let Ok(board) = fs::read_to_string(board_path) {
            let clean = board.trim();
            if !clean.is_empty() && clean != "None" && clean != "Default string" {
                let mut res = clean.to_string();
                if is_wsl && !wsl_suffix.is_empty() {
                    res = format!("{} {}", res, wsl_suffix);
                }
                return Some(res);
            }
        }
    }

    if is_wsl {
        if let Some(wsl_ver) = detect_wsl_version() {
            return Some(format!("Windows Subsystem for Linux - {}", wsl_ver));
        } else {
            return Some("Windows Subsystem for Linux".to_string());
        }
    }

    None
}

/// Detects hardware product/host model from Windows BIOS registry.
#[cfg(windows)]
pub fn detect_host() -> Option<String> {
    use crate::modules::win_util::ffi;
    let bios_key = "HARDWARE\\DESCRIPTION\\System\\BIOS";
    let product = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, bios_key, "SystemProductName")
        .or_else(|| ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, bios_key, "BaseBoardProduct"));
    let manufacturer =
        ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, bios_key, "SystemManufacturer").or_else(
            || ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, bios_key, "BaseBoardManufacturer"),
        );

    if let Some(prod) = product {
        let lower = prod.to_lowercase();
        if !lower.is_empty()
            && lower != "system product name"
            && lower != "to be filled by o.e.m."
            && lower != "default string"
            && lower != "none"
        {
            if let Some(mfg) = manufacturer {
                let mfg_lower = mfg.to_lowercase();
                if !mfg_lower.is_empty()
                    && !lower.starts_with(&mfg_lower)
                    && mfg_lower != "system manufacturer"
                {
                    return Some(format!("{} {}", mfg, prod));
                }
            }
            return Some(prod);
        }
    }
    None
}

pub struct HostCollector;

impl Collector for HostCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Host
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        detect_host().map(|host| ModuleOutput {
            id: ModuleId::Host,
            label: "Host".to_string(),
            value: host,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_os_release_ubuntu() {
        let fixture = r#"
NAME="Ubuntu"
VERSION="24.04 LTS (Noble Numbat)"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 24.04 LTS"
VERSION_ID="24.04"
"#;
        let info = parse_os_release(fixture);
        assert_eq!(info.display_name, "Ubuntu 24.04 LTS");
        assert_eq!(info.distro_id, "ubuntu");
        assert_eq!(info.distro_like, vec!["debian"]);
    }

    #[test]
    fn test_parse_os_release_arch() {
        let fixture = r#"
NAME="Arch Linux"
PRETTY_NAME="Arch Linux"
ID=arch
BUILD_ID=rolling
"#;
        let info = parse_os_release(fixture);
        assert_eq!(info.display_name, "Arch Linux");
        assert_eq!(info.distro_id, "arch");
        assert!(info.distro_like.is_empty());
    }

    #[test]
    fn test_parse_os_release_fallback_name_version() {
        let fixture = r#"
NAME="CustomOS"
VERSION="1.0"
ID=custom
"#;
        let info = parse_os_release(fixture);
        assert_eq!(info.display_name, "CustomOS 1.0");
        assert_eq!(info.distro_id, "custom");
    }

    #[test]
    fn test_parse_os_release_empty_or_corrupted() {
        let empty_info = parse_os_release("");
        assert_eq!(empty_info.display_name, "Linux");
        assert_eq!(empty_info.distro_id, "linux");
        assert!(empty_info.distro_like.is_empty());

        let corrupted = "garbage text without equals\nrandom words\n";
        let corrupt_info = parse_os_release(corrupted);
        assert_eq!(corrupt_info.display_name, "Linux");
        assert_eq!(corrupt_info.distro_id, "linux");
    }

    #[test]
    fn test_parse_os_release_unquoted() {
        let unquoted = "NAME=CustomArch\nID=arch\nID_LIKE=arch\nPRETTY_NAME=Custom Arch Linux\n";
        let info = parse_os_release(unquoted);
        assert_eq!(info.display_name, "Custom Arch Linux");
        assert_eq!(info.distro_id, "arch");
        assert_eq!(info.distro_like, vec!["arch"]);
    }

    #[test]
    fn test_parse_os_release_escaped_quotes() {
        let text = "PRETTY_NAME=\"Debian GNU/Linux 12 (\\\"Bookworm\\\")\"\nID=debian\n";
        let info = parse_os_release(text);
        assert_eq!(info.display_name, "Debian GNU/Linux 12 (\"Bookworm\")");
        assert_eq!(info.distro_id, "debian");
    }

    #[test]
    fn test_parse_windows_os_info_windows_11_upgrade() {
        // Windows 11 build >= 22000 with legacy "Windows 10 Pro" registry key
        let info = parse_windows_os_info("Windows 10 Pro", Some("23H2"), Some("22631"));
        assert_eq!(info.display_name, "Windows 11 Pro 23H2 (Build 22631)");
        assert_eq!(info.distro_id, "windows11");
        assert!(info.distro_like.is_empty());
    }

    #[test]
    fn test_parse_windows_os_info_windows_10() {
        // True Windows 10 build < 22000
        let info = parse_windows_os_info("Windows 10 Home", Some("22H2"), Some("19045"));
        assert_eq!(info.display_name, "Windows 10 Home 22H2 (Build 19045)");
        assert_eq!(info.distro_id, "windows10");
    }

    #[test]
    fn test_parse_windows_os_info_windows_7() {
        let info = parse_windows_os_info("Windows 7 Ultimate", Some("SP1"), Some("7601"));
        assert_eq!(info.display_name, "Windows 7 Ultimate SP1 (Build 7601)");
        assert_eq!(info.distro_id, "windows7");
    }

    #[test]
    fn test_parse_windows_os_info_native_win11() {
        let info = parse_windows_os_info("Windows 11 Enterprise", Some("24H2"), Some("26100"));
        assert_eq!(
            info.display_name,
            "Windows 11 Enterprise 24H2 (Build 26100)"
        );
        assert_eq!(info.distro_id, "windows11");
    }

    #[test]
    fn test_parse_windows_os_info_missing_fields() {
        let info = parse_windows_os_info("Windows 10 Pro", None, Some("22000"));
        assert_eq!(info.display_name, "Windows 11 Pro (Build 22000)");
        assert_eq!(info.distro_id, "windows11");

        let info2 = parse_windows_os_info("Windows 10", Some("21H2"), Some("19044"));
        assert_eq!(info2.display_name, "Windows 10 21H2 (Build 19044)");
        assert_eq!(info2.distro_id, "windows10");
    }

    #[test]
    fn test_parse_wsl_version_output() {
        let sample =
            "WSL version: 2.7.12.0\r\nKernel version: 6.18.33.2-2\r\nWSLg version: 1.0.73.2\r\n";
        assert_eq!(
            parse_wsl_version_output(sample),
            Some("2.7.12.0".to_string())
        );

        let sample_spaced = "WSL version : 2.4.4.0\n";
        assert_eq!(
            parse_wsl_version_output(sample_spaced),
            Some("2.4.4.0".to_string())
        );

        let none_sample = "Ubuntu Linux 24.04\n";
        assert_eq!(parse_wsl_version_output(none_sample), None);
    }
}
