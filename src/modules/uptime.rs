use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::fs;
#[cfg(not(windows))]
use std::mem::MaybeUninit;

/// Parses total uptime seconds from `/proc/uptime` content.
/// The first token represents total elapsed seconds as a floating-point number since kernel boot.
pub fn parse_uptime(content: &str) -> Option<u64> {
    let first_token = content.split_whitespace().next()?;
    let seconds_f64: f64 = first_token.parse().ok()?;
    if seconds_f64 >= 0.0 && seconds_f64.is_finite() {
        Some(seconds_f64 as u64)
    } else {
        None
    }
}

/// Formats total seconds into a readable day/hour/minute representation.
pub fn format_uptime(total_seconds: u64) -> String {
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let min_label = if minutes == 1 { "min" } else { "mins" };

    if days > 0 {
        let day_label = if days == 1 { "day" } else { "days" };
        let hour_label = if hours == 1 { "hour" } else { "hours" };
        format!(
            "{} {}, {} {}, {} {}",
            days, day_label, hours, hour_label, minutes, min_label
        )
    } else if hours > 0 {
        let hour_label = if hours == 1 { "hour" } else { "hours" };
        format!("{} {}, {} {}", hours, hour_label, minutes, min_label)
    } else {
        format!("{} {}", minutes, min_label)
    }
}

/// Reads system uptime from `/proc/uptime` with fallback to `libc::sysinfo`.
#[cfg(not(windows))]
pub fn get_uptime() -> Option<u64> {
    // Primary fast path via procfs
    if let Ok(content) = fs::read_to_string("/proc/uptime") {
        if let Some(secs) = parse_uptime(&content) {
            return Some(secs);
        }
    }

    // Fallback for chroot/container environments where /proc is unmounted or restricted
    #[cfg(target_os = "linux")]
    // SAFETY: libc::sysinfo safely writes hardware statistics into the provided uninitialized sysinfo struct pointer.
    unsafe {
        let mut info = MaybeUninit::<libc::sysinfo>::uninit();
        if libc::sysinfo(info.as_mut_ptr()) == 0 {
            let info = info.assume_init();
            if info.uptime > 0 {
                return Some(info.uptime as u64);
            }
        }
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    // SAFETY: sysctl with KERN_BOOTTIME reads kernel boot time into a valid timeval buffer, and gettimeofday reads current time.
    unsafe {
        let mut mib = [libc::CTL_KERN, libc::KERN_BOOTTIME];
        let mut boottime = MaybeUninit::<libc::timeval>::uninit();
        let mut size = std::mem::size_of::<libc::timeval>();
        if libc::sysctl(
            mib.as_mut_ptr(),
            2,
            boottime.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) == 0
        {
            let boottime = boottime.assume_init();
            let mut now = MaybeUninit::<libc::timeval>::uninit();
            if libc::gettimeofday(now.as_mut_ptr(), std::ptr::null_mut()) == 0 {
                let now = now.assume_init();
                let diff = now.tv_sec - boottime.tv_sec;
                if diff > 0 {
                    return Some(diff as u64);
                }
            }
        }
    }

    None
}

/// Reads system uptime on Windows via GetTickCount64.
#[cfg(windows)]
pub fn get_uptime() -> Option<u64> {
    // SAFETY: GetTickCount64 is a safe Windows API call that returns a u64 millisecond counter.
    unsafe {
        let ms = crate::modules::win_util::ffi::GetTickCount64();
        if ms > 0 {
            Some(ms / 1000)
        } else {
            None
        }
    }
}

pub struct UptimeCollector;

impl Collector for UptimeCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Uptime
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let secs = get_uptime()?;
        Some(ModuleOutput {
            id: ModuleId::Uptime,
            label: "Uptime".to_string(),
            value: format_uptime(secs),
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uptime_standard() {
        let fixture = "1978.59 23627.21\n";
        assert_eq!(parse_uptime(fixture), Some(1978));
    }

    #[test]
    fn test_parse_uptime_empty_or_corrupted() {
        assert_eq!(parse_uptime(""), None);
        assert_eq!(parse_uptime("   \n\t "), None);
        assert_eq!(parse_uptime("invalid text"), None);
        assert_eq!(parse_uptime("-10.5 20.0"), None);
    }

    #[test]
    fn test_parse_uptime_zero() {
        assert_eq!(parse_uptime("0.00 0.00"), Some(0));
        assert_eq!(format_uptime(0), "0 mins");
    }

    #[test]
    fn test_format_uptime_days() {
        let secs = 2 * 86400 + 5 * 3600 + 30 * 60;
        assert_eq!(format_uptime(secs), "2 days, 5 hours, 30 mins");
    }

    #[test]
    fn test_format_uptime_single_day_single_hour() {
        let secs = 86400 + 3600 + 10 * 60;
        assert_eq!(format_uptime(secs), "1 day, 1 hour, 10 mins");
    }

    #[test]
    fn test_format_uptime_hours() {
        let secs = 3 * 3600 + 15 * 60;
        assert_eq!(format_uptime(secs), "3 hours, 15 mins");
    }

    #[test]
    fn test_format_uptime_minutes() {
        let secs = 42 * 60 + 12;
        assert_eq!(format_uptime(secs), "42 mins");
    }
}
