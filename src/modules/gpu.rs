use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Maps PCI vendor hex IDs to human-readable manufacturer names.
pub fn vendor_id_to_name(vendor: &str) -> Option<&'static str> {
    let clean = vendor
        .trim()
        .strip_prefix("0x")
        .or_else(|| vendor.trim().strip_prefix("0X"))
        .unwrap_or(vendor.trim())
        .to_lowercase();

    match clean.as_str() {
        "10de" => Some("NVIDIA"),
        "1002" => Some("AMD"),
        "8086" => Some("Intel"),
        "1af4" => Some("VirtIO GPU"),
        "1414" => Some("Microsoft Direct3D"),
        "15ad" => Some("VMware SVGA"),
        "80ee" => Some("VirtualBox Graphics"),
        "1013" => Some("Cirrus Logic"),
        "1234" => Some("QEMU VGA"),
        "13d7" => Some("Broadcom VideoCore"),
        "1a03" => Some("ASPEED Graphics"),
        "102b" => Some("Matrox Graphics"),
        "1b36" => Some("Red Hat QXL"),
        "5143" => Some("Qualcomm Adreno"),
        _ => None,
    }
}

/// Cleans redundant vendor suffixes and bracketed tags from GPU names.
/// When pci.ids includes bracketed marketing names (e.g. `GA107 [GeForce RTX 2050]` or `Rembrandt [Radeon 680M]`),
/// extracts the consumer product name and formats it cleanly with the vendor prefix.
pub fn clean_gpu_name(name: &str) -> String {
    let mut cleaned = name
        .replace("(R)", "")
        .replace("(r)", "")
        .replace("(TM)", "")
        .replace("(tm)", "")
        .replace("Corporation", "")
        .replace("Technologies Inc", "")
        .replace("Advanced Micro Devices, Inc.", "AMD")
        .replace("Advanced Micro Devices", "AMD")
        .replace("[AMD/ATI]", "")
        .replace("[AMD]", "")
        .replace("[ATI]", "")
        .replace("Inc.", "")
        .replace("Inc", "");

    // Remove any revision tags like (rev a1), (rev 02), (rev 0b), etc.
    while let Some(rev_idx) = cleaned.find("(rev ") {
        if let Some(close_idx) = cleaned[rev_idx..].find(')') {
            cleaned.replace_range(rev_idx..=rev_idx + close_idx, "");
        } else {
            break;
        }
    }

    // Determine vendor prefix
    let vendor = if cleaned.contains("NVIDIA") || cleaned.contains("GeForce") {
        Some("NVIDIA")
    } else if cleaned.contains("AMD") || cleaned.contains("Radeon") {
        Some("AMD")
    } else if cleaned.contains("Intel") || cleaned.contains("Iris") || cleaned.contains("Arc") {
        Some("Intel")
    } else if cleaned.contains("Qualcomm") || cleaned.contains("Adreno") {
        Some("Qualcomm")
    } else if cleaned.contains("Apple") {
        Some("Apple")
    } else {
        None
    };

    // If there is a bracketed marketing name like `[GeForce RTX 2050]` or `[Radeon 680M]`:
    if let (Some(start), Some(end)) = (cleaned.find('['), cleaned.rfind(']')) {
        if start < end {
            let inside = cleaned[start + 1..end].trim();
            // Handle multiple slash-separated aliases like `[Radeon Vega Series / Radeon Vega Mobile Series]`
            let chosen = if let Some(first) = inside.split('/').next() {
                first.trim()
            } else {
                inside
            };

            if !chosen.is_empty() {
                let formatted = if let Some(v) = vendor {
                    if chosen.starts_with(v) {
                        chosen.to_string()
                    } else {
                        format!("{} {}", v, chosen)
                    }
                } else {
                    chosen.to_string()
                };
                let tokens: Vec<&str> = formatted.split_whitespace().collect();
                return tokens.join(" ");
            }
        }
    }

    // Fallback: strip residual brackets and normalize whitespace
    let stripped = cleaned.replace('[', "").replace(']', "");
    let tokens: Vec<&str> = stripped.split_whitespace().collect();
    tokens.join(" ")
}

/// Parses standard pci.ids file format to resolve vendor and device hex IDs to human-readable names.
pub fn parse_pci_ids_file(
    content: &str,
    target_vendor: &str,
    target_device: &str,
) -> Option<String> {
    let mut in_target_vendor = false;
    let mut vendor_name = None;

    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        if !line.starts_with('\t') {
            let mut parts = line.split_whitespace();
            if let Some(v_id) = parts.next() {
                if v_id.eq_ignore_ascii_case(target_vendor) {
                    in_target_vendor = true;
                    vendor_name = Some(parts.collect::<Vec<&str>>().join(" "));
                } else {
                    in_target_vendor = false;
                }
            }
        } else if in_target_vendor && line.starts_with('\t') && !line.starts_with("\t\t") {
            let trimmed = &line[1..];
            let mut parts = trimmed.split_whitespace();
            if let Some(d_id) = parts.next() {
                if d_id.eq_ignore_ascii_case(target_device) {
                    let dev_name = parts.collect::<Vec<&str>>().join(" ");
                    let raw = if let Some(ref v_name) = vendor_name {
                        format!("{} {}", v_name, dev_name)
                    } else {
                        dev_name
                    };
                    return Some(clean_gpu_name(&raw));
                }
            }
        }
    }

    None
}

/// Resolves PCI vendor and device hex IDs against local system pci.ids databases.
pub fn lookup_pci_ids(vendor_hex: &str, device_hex: &str) -> Option<String> {
    let vendor = vendor_hex
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_lowercase();
    let device = device_hex
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_lowercase();

    if vendor.is_empty() || device.is_empty() {
        return None;
    }

    let pci_id_paths = [
        "/usr/share/hwdata/pci.ids",
        "/usr/share/misc/pci.ids",
        "/usr/share/pci.ids",
        "/var/lib/pci.ids",
    ];

    for path in &pci_id_paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some(name) = parse_pci_ids_file(&content, &vendor, &device) {
                return Some(name);
            }
        }
    }

    None
}

/// Probes a given PCI sysfs directory for display controllers.
/// Scans for PCI base class 0x03: VGA-compatible (0x0300), 3D controller (0x0302), and display (0x0380).
pub fn detect_gpus_from_sysfs_dir(pci_dir: &Path) -> Vec<String> {
    let mut gpus = Vec::new();

    if let Ok(entries) = fs::read_dir(pci_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let class_path = path.join("class");
            if let Ok(class_str) = fs::read_to_string(class_path) {
                let class_trimmed = class_str.trim().to_lowercase();
                // Class 0x03xxxx matches display adapter devices in PCI specification
                if class_trimmed.starts_with("0x03") || class_trimmed.starts_with("3") {
                    let vendor_str = fs::read_to_string(path.join("vendor"))
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    let device_str = fs::read_to_string(path.join("device"))
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    // 1. Resolve vendor/device IDs against local pci.ids database without spawning lspci
                    if let Some(pci_name) = lookup_pci_ids(&vendor_str, &device_str) {
                        if !gpus.contains(&pci_name) {
                            gpus.push(pci_name);
                            continue;
                        }
                    }

                    // 2. Vendor mapping fallback
                    let vendor_name = vendor_id_to_name(&vendor_str);
                    let gpu_name = if let Some(v_name) = vendor_name {
                        if !device_str.is_empty() {
                            format!("{} ({})", v_name, device_str)
                        } else {
                            v_name.to_string()
                        }
                    } else if !vendor_str.is_empty() {
                        format!("PCI Display ({}:{})", vendor_str, device_str)
                    } else {
                        "Display Controller".to_string()
                    };

                    if !gpus.contains(&gpu_name) {
                        gpus.push(gpu_name);
                    }
                }
            }
        }
    }

    gpus
}

/// Probes `/sys/bus/pci/devices/` for display controllers (PCI class 0x03xxxx).
pub fn detect_gpus_sysfs() -> Vec<String> {
    detect_gpus_from_sysfs_dir(Path::new("/sys/bus/pci/devices"))
}

/// Parses the output of `lspci -mm` for display devices.
pub fn parse_lspci_mm_output(text: &str) -> Vec<String> {
    let mut gpus = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Format: Slot "Class" "Vendor" "Device" "SVendor" "SDevice"
        let parts: Vec<&str> = trimmed.split('"').collect();
        if parts.len() >= 7 {
            let vendor = parts[3].trim();
            let device = parts[5].trim();
            let raw_name = format!("{} {}", vendor, device);
            let cleaned = clean_gpu_name(&raw_name);
            let final_name = if !cleaned.is_empty() {
                cleaned
            } else {
                raw_name
            };
            if !gpus.contains(&final_name) {
                gpus.push(final_name);
            }
        }
    }
    gpus
}

/// Fallback probe using `lspci -mm` when sysfs does not yield specific models.
pub fn detect_gpus_lspci() -> Vec<String> {
    let mut gpus = Vec::new();
    for class_filter in &["::0300", "::0302", "::0380"] {
        if let Ok(output) = Command::new("lspci")
            .args(["-mm", "-d", class_filter])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let parsed = parse_lspci_mm_output(&text);
                for gpu in parsed {
                    if !gpus.contains(&gpu) {
                        gpus.push(gpu);
                    }
                }
            }
        }
    }
    gpus
}

/// Formats a GPU model name with optional VRAM and clock speed.
pub fn format_gpu_with_specs(name: &str, vram_mb: Option<u64>, clock_mhz: Option<u64>) -> String {
    let mut parts = Vec::new();
    if let Some(mb) = vram_mb {
        if mb >= 1024 {
            let gib = mb as f64 / 1024.0;
            if (gib.round() - gib).abs() < 0.05 {
                parts.push(format!("({:.0} GiB)", gib));
            } else {
                parts.push(format!("({:.1} GiB)", gib));
            }
        } else if mb > 0 {
            parts.push(format!("({} MiB)", mb));
        }
    }

    if let Some(mhz) = clock_mhz {
        if mhz >= 1000 {
            parts.push(format!("@ {:.3}GHz", mhz as f64 / 1000.0));
        } else if mhz > 0 {
            parts.push(format!("@ {}MHz", mhz));
        }
    }

    if parts.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", name, parts.join(" "))
    }
}

/// Detects GPUs when running inside Windows Subsystem for Linux (WSL2).
/// In WSL2, raw PCI devices are virtualized under Hyper-V; probes CPU iGPU tables and NVIDIA-SMI bridge.
pub fn detect_wsl_gpus() -> Vec<String> {
    let mut gpus = Vec::new();

    // 1. Probe integrated GPU (iGPU) from CPU model with typical clock and shared VRAM
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        let cpu_lower = cpuinfo.to_lowercase();
        if cpu_lower.contains("amd") {
            if cpu_lower.contains("7535hs")
                || cpu_lower.contains("6600h")
                || cpu_lower.contains("6600u")
            {
                gpus.push(format_gpu_with_specs(
                    "AMD Radeon 660M",
                    Some(512),
                    Some(1900),
                ));
            } else if cpu_lower.contains("7735hs")
                || cpu_lower.contains("6800h")
                || cpu_lower.contains("6800u")
            {
                gpus.push(format_gpu_with_specs(
                    "AMD Radeon 680M",
                    Some(512),
                    Some(2200),
                ));
            } else if cpu_lower.contains("7840hs")
                || cpu_lower.contains("8845hs")
                || cpu_lower.contains("7940hs")
            {
                gpus.push(format_gpu_with_specs(
                    "AMD Radeon 780M",
                    Some(512),
                    Some(2700),
                ));
            } else if cpu_lower.contains("radeon") {
                gpus.push("AMD Radeon Graphics".to_string());
            }
        } else if cpu_lower.contains("intel") {
            if cpu_lower.contains("iris") || cpu_lower.contains("xe") {
                gpus.push(format_gpu_with_specs(
                    "Intel Iris Xe Graphics",
                    None,
                    Some(1400),
                ));
            } else if cpu_lower.contains("uhd") {
                gpus.push(format_gpu_with_specs(
                    "Intel UHD Graphics",
                    None,
                    Some(1150),
                ));
            } else if cpu_lower.contains("hd graphics") {
                gpus.push("Intel HD Graphics".to_string());
            }
        }
    }

    // 2. Fast-path: Check persistent user/system cache for discrete GPU to achieve sub-millisecond runs
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));

    let cache_file = cache_dir.as_ref().map(|d| d.join("wsl_dgpu_v2.cache"));

    if let Some(ref path) = cache_file {
        if let Ok(cached) = fs::read_to_string(path) {
            let trimmed = cached.trim();
            if trimmed == "NONE" {
                return gpus;
            }
            if !trimmed.is_empty() && !gpus.contains(&trimmed.to_string()) {
                gpus.push(trimmed.to_string());
                return gpus;
            }
        }
    }

    // 3. Fallback: Query nvidia-smi with name, VRAM and clock speed
    let mut found_dgpu = false;
    for smi_path in &["/usr/lib/wsl/lib/nvidia-smi", "nvidia-smi"] {
        if let Ok(output) = Command::new(smi_path)
            .args([
                "--query-gpu=name,memory.total,clocks.max.graphics",
                "--format=csv,noheader",
            ])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if let Some(raw_name) = parts.first() {
                        if !raw_name.is_empty() {
                            let vram_mb = parts.get(1).and_then(|v| {
                                v.split_whitespace()
                                    .next()
                                    .and_then(|n| n.parse::<u64>().ok())
                            });
                            let clock_mhz = parts.get(2).and_then(|c| {
                                c.split_whitespace()
                                    .next()
                                    .and_then(|n| n.parse::<u64>().ok())
                            });

                            let formatted = format_gpu_with_specs(raw_name, vram_mb, clock_mhz);
                            if !gpus.contains(&formatted) {
                                gpus.push(formatted.clone());
                                found_dgpu = true;

                                // Persist discrete GPU cache
                                if let Some(ref dir) = cache_dir {
                                    let _ = fs::create_dir_all(dir);
                                }
                                if let Some(ref path) = cache_file {
                                    let _ = fs::write(path, &formatted);
                                }
                            }
                        }
                    }
                }
            }
        }
        if gpus.iter().any(|g| {
            g.contains("NVIDIA") || g.contains("GeForce") || g.contains("RTX") || g.contains("GTX")
        }) {
            break;
        }
    }

    if !found_dgpu {
        if let Some(ref dir) = cache_dir {
            let _ = fs::create_dir_all(dir);
        }
        if let Some(ref path) = cache_file {
            let _ = fs::write(path, "NONE");
        }
    }

    gpus
}

/// Classifies whether a GPU model name is an integrated graphics processor.
pub fn is_integrated_gpu(gpu_name: &str) -> bool {
    let lower = gpu_name.to_lowercase();

    // Explicit discrete GPU markers
    if lower.contains("geforce")
        || lower.contains("rtx")
        || lower.contains("gtx")
        || lower.contains("quadro")
        || lower.contains("tesla")
        || lower.contains("arc a")
        || lower.contains("arc b")
        || lower.contains("arc pro")
        || lower.contains("radeon rx")
        || lower.contains("rx ")
        || lower.contains("radeon pro")
        || lower.contains("firepro")
    {
        return false;
    }

    // Explicit integrated GPU markers
    lower.contains("660m")
        || lower.contains("680m")
        || lower.contains("760m")
        || lower.contains("780m")
        || lower.contains("740m")
        || lower.contains("880m")
        || lower.contains("890m")
        || lower.contains("radeon graphics")
        || lower.contains("radeon vega")
        || lower.contains("vega ")
        || lower.contains("iris")
        || lower.contains("uhd graphics")
        || lower.contains("hd graphics")
        || (lower.contains("intel") && !lower.contains("arc"))
        || lower.contains("adreno")
        || lower.contains("mali")
        || lower.contains("videocore")
        || lower.contains("apple m")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuGroup {
    pub name: String,
    pub count: usize,
    pub is_integrated: bool,
}

/// Groups identical dGPUs and assigns sequential dynamic indices starting at GPU0.
/// Guarantees that iGPU occupies GPU0 on hybrid laptop configurations and appends [Integrated] / [Discrete] tags.
pub fn group_and_index_gpus(raw_gpus: &[String], cpu_sockets: usize) -> Vec<ModuleOutput> {
    if raw_gpus.is_empty() {
        return Vec::new();
    }

    let mut igpus: Vec<String> = Vec::new();
    let mut dgpus: Vec<String> = Vec::new();

    for gpu in raw_gpus {
        let cleaned = clean_gpu_name(gpu);
        if is_integrated_gpu(&cleaned) {
            igpus.push(cleaned);
        } else {
            dgpus.push(cleaned);
        }
    }

    let mut groups: Vec<GpuGroup> = Vec::new();

    // 1. iGPU Handling: Takes GPU0 if present
    if !igpus.is_empty() {
        let first_igpu = igpus[0].clone();
        let socket_multiplier = if cpu_sockets > 1 { cpu_sockets } else { 1 };
        let count = igpus.len().max(socket_multiplier);
        groups.push(GpuGroup {
            name: first_igpu,
            count,
            is_integrated: true,
        });
    }

    // 2. dGPU Grouping & Ordering: Process in detection order and group identical dGPUs
    for dgpu in dgpus {
        if let Some(existing) = groups
            .iter_mut()
            .find(|g| !g.is_integrated && g.name == dgpu)
        {
            existing.count += 1;
        } else {
            groups.push(GpuGroup {
                name: dgpu,
                count: 1,
                is_integrated: false,
            });
        }
    }

    // 3. Format into ModuleOutput with sequential indices (GPU0, GPU1, ...)
    let mut outputs = Vec::new();
    for (idx, group) in groups.iter().enumerate() {
        let label = format!("GPU{}", idx);
        let tag = if group.is_integrated {
            "[Integrated]"
        } else {
            "[Discrete]"
        };

        let formatted_name =
            if group.name.contains("[Integrated]") || group.name.contains("[Discrete]") {
                group.name.clone()
            } else {
                format!("{} {}", group.name, tag)
            };

        let value = if group.count > 1 {
            format!("{}x {}", group.count, formatted_name)
        } else {
            formatted_name
        };

        outputs.push(ModuleOutput {
            id: ModuleId::Gpu,
            label,
            value,
            custom_rendered: None,
        });
    }

    outputs
}

/// Probes maximum graphics clock frequency (in MHz) from sysfs DRM and hwmon entries.
pub fn detect_gpu_clock_mhz(card_idx: usize) -> Option<u64> {
    let card_dir = format!("/sys/class/drm/card{}", card_idx);
    // 1. Intel DRM gt frequencies (gt_max_freq_mhz, gt_boost_freq_mhz, gt_RP0_freq_mhz)
    for file in &["gt_max_freq_mhz", "gt_boost_freq_mhz", "gt_RP0_freq_mhz"] {
        if let Ok(val) = fs::read_to_string(format!("{}/{}", card_dir, file)) {
            if let Ok(mhz) = val.trim().parse::<u64>() {
                if mhz > 0 {
                    return Some(mhz);
                }
            }
        }
    }
    // 2. AMD / Generic hwmon freq1_max
    if let Ok(entries) = fs::read_dir(format!("{}/device/hwmon", card_dir)) {
        for entry in entries.flatten() {
            if let Ok(val) = fs::read_to_string(entry.path().join("freq1_max")) {
                if let Ok(hz) = val.trim().parse::<u64>() {
                    let mhz = if hz > 1_000_000 { hz / 1_000_000 } else { hz };
                    if mhz > 0 {
                        return Some(mhz);
                    }
                }
            }
        }
    }
    None
}

/// Parses and filters raw display adapter DriverDesc strings from Windows registry.
pub fn parse_windows_gpu_names(adapters: &[String]) -> Vec<String> {
    let mut gpus = Vec::new();
    for desc in adapters {
        let clean = clean_gpu_name(desc);
        let lower = clean.to_lowercase();
        if !lower.is_empty()
            && !lower.contains("rdpdd chained dd")
            && !lower.contains("mirage driver")
            && !lower.contains("remote desktop")
            && !lower.contains("indirect display")
            && !gpus.contains(&clean)
        {
            gpus.push(clean);
        }
    }
    gpus
}

#[cfg(not(windows))]
static GPU_CACHE: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

#[cfg(not(windows))]
pub fn get_gpu_list() -> Vec<String> {
    GPU_CACHE.get_or_init(get_gpu_list_uncached).clone()
}

#[cfg(not(windows))]
fn get_gpu_list_uncached() -> Vec<String> {
    // 0. Fast-path: Check persistent cache
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));

    let cache_file = cache_dir.as_ref().map(|d| d.join("gpu_list_v1.cache"));

    if let Some(ref path) = cache_file {
        if let Ok(content) = fs::read_to_string(path) {
            let list: Vec<String> = content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            if !list.is_empty() {
                return list;
            }
        }
    }

    let sysfs_gpus = detect_gpus_sysfs();

    // Check if sysfs produced only generic/unresolved IDs (e.g. "Intel (0x5917)" or "Microsoft Direct3D")
    let is_incomplete = sysfs_gpus.is_empty()
        || sysfs_gpus.iter().any(|g| {
            g.contains("0x")
                || g.contains("PCI Display")
                || g.contains("Display Controller")
                || g.contains("Microsoft Direct3D")
                || g == "Intel"
                || g == "NVIDIA"
                || g == "AMD"
                || g.starts_with("Onboard")
        });

    let raw_list = if is_incomplete {
        // If in WSL or Direct3D detected, probe WSL GPU bridge (iGPU + dGPU)
        let wsl_gpus = detect_wsl_gpus();
        if !wsl_gpus.is_empty() {
            wsl_gpus
        } else {
            let lspci_gpus = detect_gpus_lspci();
            if !lspci_gpus.is_empty() {
                lspci_gpus
            } else {
                sysfs_gpus
            }
        }
    } else {
        sysfs_gpus
    };

    let result: Vec<String> = raw_list
        .into_iter()
        .enumerate()
        .map(|(idx, gpu)| {
            if !gpu.contains('@') {
                if let Some(mhz) = detect_gpu_clock_mhz(idx) {
                    return format_gpu_with_specs(&gpu, None, Some(mhz));
                }
            }
            gpu
        })
        .collect();

    if !result.is_empty() {
        if let Some(ref dir) = cache_dir {
            let _ = fs::create_dir_all(dir);
        }
        if let Some(ref path) = cache_file {
            let _ = fs::write(path, result.join("\n"));
        }
    }

    result
}

/// Reads display adapter names from Windows registry under Display Class GUID.
#[cfg(windows)]
pub fn get_gpu_list() -> Vec<String> {
    use crate::modules::win_util::ffi;
    let class_key =
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let subkeys = ffi::reg_enum_subkeys(ffi::HKEY_LOCAL_MACHINE, class_key);

    let mut raw_adapters = Vec::new();
    for sub in subkeys {
        if sub.chars().all(|c| c.is_ascii_digit()) {
            let key = format!("{}\\{}", class_key, sub);
            if let Some(desc) = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, &key, "DriverDesc") {
                raw_adapters.push(desc);
            }
        }
    }

    let parsed = parse_windows_gpu_names(&raw_adapters);
    if !parsed.is_empty() {
        parsed
    } else {
        vec!["Display Adapter".to_string()]
    }
}

pub fn get_gpu_info() -> Option<String> {
    let list = get_gpu_list();
    let cpu_sockets = crate::modules::cpu::get_cpu_info()
        .map(|c| c.sockets)
        .unwrap_or(1);
    let outputs = group_and_index_gpus(&list, cpu_sockets);
    if outputs.is_empty() {
        None
    } else {
        Some(
            outputs
                .into_iter()
                .map(|o| format!("{}: {}", o.label, o.value))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

pub struct GpuCollector;

impl Collector for GpuCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Gpu
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let value = get_gpu_info()?;
        Some(ModuleOutput {
            id: ModuleId::Gpu,
            label: "GPU".to_string(),
            value,
            custom_rendered: None,
        })
    }

    fn collect_multiple(&self, _ctx: &FetchContext) -> Vec<ModuleOutput> {
        let gpus = get_gpu_list();
        let cpu_sockets = crate::modules::cpu::get_cpu_info()
            .map(|c| c.sockets)
            .unwrap_or(1);
        group_and_index_gpus(&gpus, cpu_sockets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id_mapping() {
        assert_eq!(vendor_id_to_name("0x10de"), Some("NVIDIA"));
        assert_eq!(vendor_id_to_name("10DE"), Some("NVIDIA"));
        assert_eq!(vendor_id_to_name("0x1002"), Some("AMD"));
        assert_eq!(vendor_id_to_name("0x8086"), Some("Intel"));
        assert_eq!(vendor_id_to_name("0x1414"), Some("Microsoft Direct3D"));
        assert_eq!(vendor_id_to_name("0x1af4"), Some("VirtIO GPU"));
        assert_eq!(vendor_id_to_name("0x15ad"), Some("VMware SVGA"));
        assert_eq!(vendor_id_to_name("0x9999"), None);
    }

    #[test]
    fn test_parse_lspci_mm_output() {
        let text = r#"
00:02.0 "VGA compatible controller" "Intel Corporation" "CometLake-H GT2 [UHD Graphics]" -r02 "Dell" "Device 099f"
01:00.0 "3D controller" "NVIDIA Corporation" "TU117M [GeForce GTX 1650 Ti Mobile]" -ra1 "Dell" "Device 099f"
"#;
        let parsed = parse_lspci_mm_output(text);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], "Intel UHD Graphics");
        assert_eq!(parsed[1], "NVIDIA GeForce GTX 1650 Ti Mobile");
    }

    #[test]
    fn test_parse_pci_ids_file() {
        let sample = r#"
# PCI IDs Sample
8086  Intel Corporation
	5917  UHD Graphics 620
	3e92  CoffeeLake-S GT2 [UHD Graphics 630]
10de  NVIDIA Corporation
	1f95  TU117M [GeForce GTX 1650 Ti Mobile]
"#;
        assert_eq!(
            parse_pci_ids_file(sample, "8086", "5917"),
            Some("Intel UHD Graphics 620".to_string())
        );
        assert_eq!(
            parse_pci_ids_file(sample, "8086", "3e92"),
            Some("Intel UHD Graphics 630".to_string())
        );
        assert_eq!(
            parse_pci_ids_file(sample, "10de", "1f95"),
            Some("NVIDIA GeForce GTX 1650 Ti Mobile".to_string())
        );
        assert_eq!(parse_pci_ids_file(sample, "8086", "9999"), None);
    }

    #[test]
    fn test_clean_gpu_name() {
        assert_eq!(
            clean_gpu_name("Intel Corporation UHD Graphics 620 (rev 07)"),
            "Intel UHD Graphics 620"
        );
        assert_eq!(
            clean_gpu_name("NVIDIA Corporation GA106 [GeForce RTX 3060]"),
            "NVIDIA GeForce RTX 3060"
        );
        assert_eq!(
            clean_gpu_name("Advanced Micro Devices, Inc. [AMD/ATI] Navi 22 [Radeon RX 6700 XT]"),
            "AMD Radeon RX 6700 XT"
        );
        assert_eq!(
            clean_gpu_name("Advanced Micro Devices, Inc. [AMD/ATI] Rembrandt [Radeon 680M] (rev 0b)"),
            "AMD Radeon 680M"
        );
        assert_eq!(
            clean_gpu_name("NVIDIA Corporation GA107 [GeForce RTX 2050] (rev a1)"),
            "NVIDIA GeForce RTX 2050"
        );
    }

    #[test]
    fn test_detect_gpus_from_sysfs_dir_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pci_dir = temp_dir.path();

        // GPU 1: Intel
        let gpu1 = pci_dir.join("0000:00:02.0");
        fs::create_dir_all(&gpu1).unwrap();
        fs::write(gpu1.join("class"), "0x030000\n").unwrap();
        fs::write(gpu1.join("vendor"), "0x8086\n").unwrap();
        fs::write(gpu1.join("device"), "0x9bc4\n").unwrap();

        // Non-display device (Network)
        let net = pci_dir.join("0000:02:00.0");
        fs::create_dir_all(&net).unwrap();
        fs::write(net.join("class"), "0x020000\n").unwrap();
        fs::write(net.join("vendor"), "0x8086\n").unwrap();

        let gpus = detect_gpus_from_sysfs_dir(pci_dir);
        assert_eq!(gpus.len(), 1);
        assert!(gpus[0].contains("Intel"));
    }

    #[test]
    fn test_virtio_and_hyperv_sysfs_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pci_dir = temp_dir.path();

        let virtio = pci_dir.join("0000:00:01.0");
        fs::create_dir_all(&virtio).unwrap();
        fs::write(virtio.join("class"), "0x030000\n").unwrap();
        fs::write(virtio.join("vendor"), "0x1af4\n").unwrap();

        let gpus = detect_gpus_from_sysfs_dir(pci_dir);
        assert_eq!(gpus, vec!["VirtIO GPU".to_string()]);
    }

    #[test]
    fn test_wsl_gpu_detection_live() {
        // In WSL2 environment, detect_wsl_gpus should succeed without panic
        let wsl_gpus = detect_wsl_gpus();
        // If run on this machine or in CI, check that returned strings are non-empty
        for gpu in wsl_gpus {
            assert!(!gpu.trim().is_empty());
        }
    }

    #[test]
    fn test_group_and_index_gpus_senior_dev_spec() {
        // 3 identical CPUs with iGPUs, 2 identical dGPUs, and 2 distinct dGPUs
        let gpus = vec![
            "AMD Radeon Graphics".to_string(),
            "NVIDIA GeForce RTX 4090".to_string(),
            "NVIDIA GeForce RTX 4090".to_string(),
            "NVIDIA GeForce RTX 3090".to_string(),
            "Intel Arc A770".to_string(),
        ];
        let outputs = group_and_index_gpus(&gpus, 3);
        assert_eq!(outputs.len(), 4);
        assert_eq!(outputs[0].label, "GPU0");
        assert_eq!(outputs[0].value, "3x AMD Radeon Graphics [Integrated]");
        assert_eq!(outputs[1].label, "GPU1");
        assert_eq!(outputs[1].value, "2x NVIDIA GeForce RTX 4090 [Discrete]");
        assert_eq!(outputs[2].label, "GPU2");
        assert_eq!(outputs[2].value, "NVIDIA GeForce RTX 3090 [Discrete]");
        assert_eq!(outputs[3].label, "GPU3");
        assert_eq!(outputs[3].value, "Intel Arc A770 [Discrete]");
    }

    #[test]
    fn test_group_and_index_gpus_no_igpu() {
        let gpus = vec![
            "NVIDIA GeForce RTX 3080".to_string(),
            "NVIDIA GeForce RTX 3080".to_string(),
        ];
        let outputs = group_and_index_gpus(&gpus, 1);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].label, "GPU0");
        assert_eq!(outputs[0].value, "2x NVIDIA GeForce RTX 3080 [Discrete]");
    }

    #[test]
    fn test_group_and_index_gpus_single_gpu() {
        let gpus = vec!["NVIDIA GeForce RTX 3060".to_string()];
        let outputs = group_and_index_gpus(&gpus, 1);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].label, "GPU0");
        assert_eq!(outputs[0].value, "NVIDIA GeForce RTX 3060 [Discrete]");
    }

    #[test]
    fn test_group_and_index_gpus_hybrid_laptop() {
        let gpus = vec![
            "Intel Iris Xe Graphics".to_string(),
            "NVIDIA GeForce RTX 4070".to_string(),
        ];
        let outputs = group_and_index_gpus(&gpus, 1);
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].label, "GPU0");
        assert_eq!(outputs[0].value, "Intel Iris Xe Graphics [Integrated]");
        assert_eq!(outputs[1].label, "GPU1");
        assert_eq!(outputs[1].value, "NVIDIA GeForce RTX 4070 [Discrete]");
    }

    #[test]
    fn test_parse_windows_gpu_names() {
        let raw = vec![
            "NVIDIA GeForce RTX 4080".to_string(),
            "Intel(R) UHD Graphics 770".to_string(),
            "RDPDD Chained DD".to_string(),
            "Mirage Driver".to_string(),
            "NVIDIA GeForce RTX 4080".to_string(),
        ];
        let parsed = parse_windows_gpu_names(&raw);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], "NVIDIA GeForce RTX 4080");
        assert_eq!(parsed[1], "Intel UHD Graphics 770");
    }
}
