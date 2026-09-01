use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
use std::collections::HashSet;
#[cfg(unix)]
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub physical_cores: usize,
    pub sockets: usize,
    pub freq_ghz: Option<f64>,
    pub max_freq_ghz: Option<f64>,
}

/// Cleans redundant marketing and frequency tokens from raw CPU model strings.
pub fn clean_cpu_model(raw: &str) -> String {
    let mut cleaned = raw
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("(tm)", "")
        .replace("CPU", "")
        .replace("Processor", "")
        .replace("with Radeon Graphics", "")
        .replace("with Radeon Vega Graphics", "")
        .replace("with Intel UHD Graphics", "")
        .replace("with Intel HD Graphics", "")
        .replace("with Intel Iris Xe Graphics", "")
        .replace("Dual-Core", "")
        .replace("Quad-Core", "")
        .replace("Six-Core", "")
        .replace("Eight-Core", "")
        .replace("12-Core", "")
        .replace("16-Core", "")
        .replace("24-Core", "")
        .replace("32-Core", "")
        .replace("64-Core", "")
        .replace("128-Core", "");

    // Strip clock speed patterns like "@ 2.60GHz" (handled dynamically via cpufreq)
    if let Some(idx) = cleaned.find('@') {
        cleaned = cleaned[..idx].to_string();
    }

    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    tokens.join(" ").trim_end_matches(',').trim().to_string()
}

/// Parses `/proc/cpuinfo` into model, logical core count, physical core count, socket count, and frequencies.
pub fn parse_cpu_info(content: &str) -> Option<CpuInfo> {
    if content.trim().is_empty() {
        return None;
    }

    let mut model_name: Option<String> = None;
    let mut processor_count = 0;
    let mut physical_ids = HashSet::new();
    let mut core_pairs = HashSet::new();
    let mut current_phys_id: Option<usize> = None;
    let mut current_core_id: Option<usize> = None;
    let mut cpu_cores_per_pkg: Option<usize> = None;
    let mut live_freq_ghz: Option<f64> = None;
    let mut rated_max_ghz: Option<f64> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if let (Some(p), Some(c)) = (current_phys_id, current_core_id) {
                core_pairs.insert((p, c));
            }
            current_phys_id = None;
            current_core_id = None;
            continue;
        }

        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim();

            match key.as_str() {
                "model name" | "model_name" | "hardware" | "cpu model" => {
                    if model_name.is_none() && !val.is_empty() {
                        model_name = Some(val.to_string());
                        if rated_max_ghz.is_none() {
                            if let Some(at_idx) = val.find('@') {
                                let after_at = val[at_idx + 1..].trim();
                                if let Some(ghz_idx) = after_at.to_lowercase().find("ghz") {
                                    if let Ok(ghz_val) = after_at[..ghz_idx].trim().parse::<f64>() {
                                        if ghz_val > 0.0 {
                                            rated_max_ghz = Some(ghz_val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "cpu mhz" | "cpu_mhz" | "clock" => {
                    if live_freq_ghz.is_none() {
                        if let Ok(mhz) = val.parse::<f64>() {
                            if mhz > 0.0 {
                                live_freq_ghz = Some(mhz / 1000.0);
                            }
                        }
                    }
                }
                "cpu" => {
                    // PowerPC / POWER architectures (e.g. "cpu : POWER9, altivec supported")
                    if model_name.is_none()
                        && !val.is_empty()
                        && val.chars().any(|c| c.is_alphabetic())
                    {
                        model_name = Some(val.to_string());
                    }
                }
                "processor" => {
                    if val.chars().all(|c| c.is_ascii_digit()) {
                        processor_count += 1;
                    } else if model_name.is_none() && !val.is_empty() {
                        model_name = Some(val.to_string());
                    }
                }
                "model" => {
                    // On ARM/devicetree, 'Model' is the hardware board/CPU name (contains non-digits)
                    if model_name.is_none()
                        && !val.is_empty()
                        && val.chars().any(|c| c.is_alphabetic())
                    {
                        model_name = Some(val.to_string());
                    }
                }
                "physical id" | "physical_id" => {
                    if let Ok(id) = val.parse::<usize>() {
                        physical_ids.insert(id);
                        current_phys_id = Some(id);
                    }
                }
                "core id" | "core_id" => {
                    if let Ok(id) = val.parse::<usize>() {
                        current_core_id = Some(id);
                    }
                }
                "cpu cores" if cpu_cores_per_pkg.is_none() => {
                    if let Ok(c) = val.parse::<usize>() {
                        if c > 0 {
                            cpu_cores_per_pkg = Some(c);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if let (Some(p), Some(c)) = (current_phys_id, current_core_id) {
        core_pairs.insert((p, c));
    }

    if model_name.is_none() && processor_count == 0 {
        return None;
    }

    let model = model_name.unwrap_or_else(|| "Unknown CPU".to_string());
    // Fallback to POSIX sysconf when processor stanzas are missing or masked in containers
    let cores = if processor_count > 0 {
        processor_count
    } else {
        #[cfg(unix)]
        // SAFETY: libc::sysconf is safe as it queries system constants and returns an integer without mutating memory.
        unsafe {
            let n = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
            if n > 0 {
                n as usize
            } else {
                1
            }
        }
        #[cfg(not(unix))]
        {
            1
        }
    };

    // Distinct physical id count indicates multi-socket server topology
    let sockets = if !physical_ids.is_empty() {
        physical_ids.len()
    } else {
        1
    };

    let physical_cores = if !core_pairs.is_empty() {
        core_pairs.len()
    } else if let Some(per_pkg) = cpu_cores_per_pkg {
        per_pkg * sockets
    } else {
        cores
    };

    Some(CpuInfo {
        model,
        cores,
        physical_cores,
        sockets,
        freq_ghz: live_freq_ghz,
        max_freq_ghz: rated_max_ghz,
    })
}

/// Formats Windows CPU registry values into a structured CpuInfo.
pub fn format_windows_cpu_info(model: &str, mhz: Option<u32>, cores: usize) -> CpuInfo {
    let freq_ghz = mhz.map(|m| m as f64 / 1000.0);
    CpuInfo {
        model: model.to_string(),
        cores: if cores > 0 { cores } else { 1 },
        physical_cores: if cores > 0 { cores } else { 1 },
        sockets: 1,
        freq_ghz,
        max_freq_ghz: freq_ghz,
    }
}

/// Fallback cpufreq sysfs reader for live and max frequencies.
#[cfg(not(windows))]
pub fn get_cpu_freq_pair_ghz() -> (Option<f64>, Option<f64>) {
    let mut cur = None;
    let mut max = None;

    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
    {
        if let Ok(khz) = content.trim().parse::<f64>() {
            if khz > 0.0 {
                cur = Some(khz / 1_000_000.0);
            }
        }
    }

    for path in &[
        "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq",
        "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
    ] {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(khz) = content.trim().parse::<f64>() {
                if khz > 0.0 {
                    max = Some(khz / 1_000_000.0);
                    break;
                }
            }
        }
    }

    (cur, max)
}

#[cfg(not(windows))]
pub fn get_cpu_freq_ghz() -> Option<f64> {
    get_cpu_freq_pair_ghz()
        .0
        .or_else(|| get_cpu_freq_pair_ghz().1)
}

#[cfg(not(windows))]
pub fn get_cpu_info() -> Option<CpuInfo> {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        if let Some(mut info) = parse_cpu_info(&content) {
            let (sys_cur, sys_max) = get_cpu_freq_pair_ghz();
            if sys_cur.is_some() {
                info.freq_ghz = sys_cur;
            }
            if sys_max.is_some() {
                info.max_freq_ghz = sys_max;
            }
            return Some(info);
        }
    }
    None
}

/// Queries Windows registry for CPU model name, clock speed, and core count.
#[cfg(windows)]
pub fn get_cpu_info() -> Option<CpuInfo> {
    use crate::modules::win_util::ffi;
    let cpu0_key = "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0";
    let model = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, cpu0_key, "ProcessorNameString")
        .unwrap_or_else(|| "Unknown CPU".to_string());
    let mhz = ffi::reg_read_u32(ffi::HKEY_LOCAL_MACHINE, cpu0_key, "~MHz");

    let subkeys = ffi::reg_enum_subkeys(
        ffi::HKEY_LOCAL_MACHINE,
        "HARDWARE\\DESCRIPTION\\System\\CentralProcessor",
    );
    let cores = if !subkeys.is_empty() {
        subkeys.len()
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    };

    Some(format_windows_cpu_info(&model, mhz, cores))
}

#[cfg(windows)]
pub fn get_cpu_freq_ghz() -> Option<f64> {
    use crate::modules::win_util::ffi;
    let cpu0_key = "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0";
    ffi::reg_read_u32(ffi::HKEY_LOCAL_MACHINE, cpu0_key, "~MHz").map(|m| m as f64 / 1000.0)
}

pub struct CpuCollector;

impl Collector for CpuCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Cpu
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let cpu = get_cpu_info()?;
        let cleaned = clean_cpu_model(&cpu.model);

        // Core / thread layout formatting
        let core_str = if cpu.physical_cores > 0 && cpu.physical_cores != cpu.cores {
            format!("({}c {}t)", cpu.physical_cores, cpu.cores)
        } else if cpu.cores > 0 {
            format!("({})", cpu.cores)
        } else {
            String::new()
        };

        // Dual frequency formatting: instantaneous live clock + rated max boost clock
        let freq_str = match (cpu.freq_ghz, cpu.max_freq_ghz) {
            (Some(live), Some(max)) => {
                if (live - max).abs() > 0.05 {
                    format!(" @ {:.3}GHz [{:.2}GHz max]", live, max)
                } else {
                    format!(" @ {:.3}GHz", live)
                }
            }
            (Some(live), None) => format!(" @ {:.3}GHz", live),
            (None, Some(max)) => format!(" @ {:.2}GHz", max),
            (None, None) => String::new(),
        };

        let mut parts = Vec::new();
        if cpu.sockets > 1 {
            parts.push(format!("{}x", cpu.sockets));
        }
        parts.push(cleaned);
        if !core_str.is_empty() {
            parts.push(core_str);
        }

        let value = format!("{}{}", parts.join(" "), freq_str);

        Some(ModuleOutput {
            id: ModuleId::Cpu,
            label: "CPU".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_cpu_model_intel() {
        let raw = "Intel(R) Core(TM) i7-10750H CPU @ 2.60GHz";
        assert_eq!(clean_cpu_model(raw), "Intel Core i7-10750H");
    }

    #[test]
    fn test_clean_cpu_model_amd() {
        let raw = "AMD Ryzen 7 7700X with Radeon Graphics";
        assert_eq!(clean_cpu_model(raw), "AMD Ryzen 7 7700X");
    }

    #[test]
    fn test_parse_cpu_info_standard() {
        let fixture = r#"
processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model name	: Intel(R) Core(TM) i7-10750H CPU @ 2.60GHz
physical id	: 0

processor	: 1
vendor_id	: GenuineIntel
cpu family	: 6
model name	: Intel(R) Core(TM) i7-10750H CPU @ 2.60GHz
physical id	: 0
"#;
        let info = parse_cpu_info(fixture).unwrap();
        assert_eq!(info.cores, 2);
        assert_eq!(info.sockets, 1);
        assert_eq!(clean_cpu_model(&info.model), "Intel Core i7-10750H");
    }

    #[test]
    fn test_parse_cpu_info_empty() {
        assert_eq!(parse_cpu_info(""), None);
        assert_eq!(parse_cpu_info("   \n\n  \t "), None);
    }

    #[test]
    fn test_parse_cpu_info_malformed() {
        let malformed = "some_random_key: some_value\nanother_line\n";
        assert_eq!(parse_cpu_info(malformed), None);
    }

    #[test]
    fn test_parse_cpu_info_dual_socket() {
        let fixture = r#"
processor	: 0
model name	: Intel(R) Xeon(R) Gold 6248R CPU @ 3.00GHz
physical id	: 0

processor	: 1
model name	: Intel(R) Xeon(R) Gold 6248R CPU @ 3.00GHz
physical id	: 1
"#;
        let info = parse_cpu_info(fixture).unwrap();
        assert_eq!(info.cores, 2);
        assert_eq!(info.sockets, 2);
        assert_eq!(clean_cpu_model(&info.model), "Intel Xeon Gold 6248R");
    }

    #[test]
    fn test_clean_cpu_model_multi_core_tokens() {
        let raw = "AMD EPYC 7763 64-Core Processor";
        assert_eq!(clean_cpu_model(raw), "AMD EPYC 7763");
    }

    #[test]
    fn test_format_windows_cpu_info() {
        let info = format_windows_cpu_info("12th Gen Intel(R) Core(TM) i7-12700K", Some(3600), 20);
        assert_eq!(info.model, "12th Gen Intel(R) Core(TM) i7-12700K");
        assert_eq!(info.cores, 20);
        assert_eq!(info.physical_cores, 20);
        assert_eq!(info.sockets, 1);
        assert_eq!(info.freq_ghz, Some(3.6));
        assert_eq!(
            clean_cpu_model(&info.model),
            "12th Gen Intel Core i7-12700K"
        );
    }

    #[test]
    fn test_parse_cpu_info_cores_vs_threads() {
        let fixture = r#"
processor	: 0
model name	: AMD Ryzen 5 7535HS with Radeon Graphics
physical id	: 0
core id		: 0
cpu cores	: 6

processor	: 1
model name	: AMD Ryzen 5 7535HS with Radeon Graphics
physical id	: 0
core id		: 0
cpu cores	: 6

processor	: 2
model name	: AMD Ryzen 5 7535HS with Radeon Graphics
physical id	: 0
core id		: 1
cpu cores	: 6
"#;
        let info = parse_cpu_info(fixture).unwrap();
        assert_eq!(info.cores, 3);
        assert_eq!(info.physical_cores, 2); // 2 distinct core IDs: 0 and 1
        assert_eq!(clean_cpu_model(&info.model), "AMD Ryzen 5 7535HS");
    }
}
