use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::ffi::CStr;
#[cfg(not(windows))]
use std::mem::MaybeUninit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnameInfo {
    pub sysname: String,
    pub hostname: String,
    pub kernel_release: String,
    pub architecture: String,
}

/// Parses and formats Windows NT kernel version string.
pub fn parse_windows_kernel_info(
    current_version: Option<&str>,
    build_number: Option<&str>,
) -> String {
    match (current_version, build_number) {
        (Some(ver), Some(build)) if !ver.trim().is_empty() && !build.trim().is_empty() => {
            let ver_clean = ver.trim();
            let build_clean = build.trim();
            if build_clean.starts_with(ver_clean) {
                format!("Windows NT {}", build_clean)
            } else {
                format!("Windows NT {}.{}", ver_clean, build_clean)
            }
        }
        (Some(ver), None) if !ver.trim().is_empty() => format!("Windows NT {}", ver.trim()),
        (None, Some(build)) if !build.trim().is_empty() => format!("Windows NT {}", build.trim()),
        _ => "Windows NT".to_string(),
    }
}

/// Retrieves POSIX utsname system metadata via direct libc uname syscall.
/// Avoids spawning subprocesses (`uname -r`) and parsing `/proc/version` directly.
#[cfg(not(windows))]
pub fn get_uname_info() -> Option<UnameInfo> {
    // SAFETY: uts points to valid uninitialized storage for struct libc::utsname, which
    // libc::uname populates on success (returning 0). Strings in uts are null-terminated.
    unsafe {
        let mut uts = MaybeUninit::<libc::utsname>::uninit();
        if libc::uname(uts.as_mut_ptr()) != 0 {
            return None;
        }
        let uts = uts.assume_init();

        let sysname = CStr::from_ptr(uts.sysname.as_ptr())
            .to_string_lossy()
            .into_owned();
        let hostname = CStr::from_ptr(uts.nodename.as_ptr())
            .to_string_lossy()
            .into_owned();
        let kernel_release = CStr::from_ptr(uts.release.as_ptr())
            .to_string_lossy()
            .into_owned();
        let architecture = CStr::from_ptr(uts.machine.as_ptr())
            .to_string_lossy()
            .into_owned();

        Some(UnameInfo {
            sysname,
            hostname,
            kernel_release,
            architecture,
        })
    }
}

/// Retrieves Windows NT system and kernel metadata from the registry and environment.
#[cfg(windows)]
pub fn get_uname_info() -> Option<UnameInfo> {
    use crate::modules::win_util::ffi;
    let key = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let major = ffi::reg_read_u32(ffi::HKEY_LOCAL_MACHINE, key, "CurrentMajorVersionNumber");
    let minor = ffi::reg_read_u32(ffi::HKEY_LOCAL_MACHINE, key, "CurrentMinorVersionNumber");
    let current_version = if let (Some(maj), Some(min)) = (major, minor) {
        Some(format!("{}.{}", maj, min))
    } else {
        ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "CurrentVersion")
    };
    let build_number = ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "CurrentBuildNumber")
        .or_else(|| ffi::reg_read_string(ffi::HKEY_LOCAL_MACHINE, key, "CurrentBuild"));

    let kernel_release =
        parse_windows_kernel_info(current_version.as_deref(), build_number.as_deref());

    let architecture =
        std::env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "x86_64".to_string());
    let arch_clean = match architecture.to_lowercase().as_str() {
        "amd64" => "x86_64".to_string(),
        "arm64" => "aarch64".to_string(),
        "x86" => "i686".to_string(),
        other => other.to_string(),
    };

    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string());

    Some(UnameInfo {
        sysname: "Windows NT".to_string(),
        hostname,
        kernel_release,
        architecture: arch_clean,
    })
}

pub struct KernelCollector;

impl Collector for KernelCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Kernel
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        let uname = ctx.uname_info.as_ref().cloned().or_else(get_uname_info)?;
        Some(ModuleOutput {
            id: ModuleId::Kernel,
            label: "Kernel".to_string(),
            value: uname.kernel_release,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uname_info_live() {
        let uname = get_uname_info();
        assert!(uname.is_some());
        let info = uname.unwrap();
        assert!(!info.kernel_release.is_empty());
        assert!(!info.architecture.is_empty());
    }

    #[test]
    fn test_parse_windows_kernel_info() {
        assert_eq!(
            parse_windows_kernel_info(Some("10.0"), Some("22631")),
            "Windows NT 10.0.22631"
        );
        assert_eq!(
            parse_windows_kernel_info(Some("6.3"), Some("9600")),
            "Windows NT 6.3.9600"
        );
        assert_eq!(
            parse_windows_kernel_info(None, Some("22000")),
            "Windows NT 22000"
        );
        assert_eq!(
            parse_windows_kernel_info(Some("10.0"), None),
            "Windows NT 10.0"
        );
        assert_eq!(parse_windows_kernel_info(None, None), "Windows NT");
    }
}
