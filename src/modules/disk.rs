//! Storage Partition & Filesystem Type Collector
//!
//! Enumerates mounted storage partitions across POSIX and Windows filesystems via `statvfs`
//! and `GetDiskFreeSpaceExW`. Displays partition capacity, percentage, and filesystem types
//! (`ext4`, `btrfs`, `ntfs`, `9p`, `vfat`, `zfs`), suggested by @Laynsb.

use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
#[cfg(not(windows))]
use std::ffi::CString;
#[cfg(not(windows))]
use std::mem::MaybeUninit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskUsage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage: u8,
}

/// Queries filesystem storage capacity and usage via POSIX statvfs syscall.
#[cfg(not(windows))]
#[allow(clippy::unnecessary_cast)]
pub fn get_disk_usage(path: &str) -> Option<DiskUsage> {
    let c_path = CString::new(path).ok()?;
    // SAFETY: c_path is a valid null-terminated C string. stat is an uninitialized
    // libc::statvfs struct that statvfs initializes on success (return code 0).
    unsafe {
        let mut stat = MaybeUninit::<libc::statvfs>::uninit();
        if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) != 0 {
            return None;
        }
        let stat = stat.assume_init();

        // `f_frsize` (fundamental block size) determines true byte capacity; fallback to `f_bsize` if 0
        let block_size = if stat.f_frsize > 0 {
            stat.f_frsize as u64
        } else {
            stat.f_bsize as u64
        };

        let total_bytes = (stat.f_blocks as u64).saturating_mul(block_size);
        let free_bytes = (stat.f_bavail as u64).saturating_mul(block_size);
        let used_bytes =
            total_bytes.saturating_sub((stat.f_bfree as u64).saturating_mul(block_size));

        if total_bytes == 0 {
            return None;
        }

        let percentage = ((used_bytes as f64 / total_bytes as f64) * 100.0)
            .round()
            .min(100.0) as u8;

        Some(DiskUsage {
            total_bytes,
            used_bytes,
            free_bytes,
            percentage,
        })
    }
}

/// Queries filesystem storage capacity and usage on Windows via GetDiskFreeSpaceExW.
#[cfg(windows)]
pub fn get_disk_usage(path: &str) -> Option<DiskUsage> {
    let path_str = if path.is_empty() || path == "/" {
        "C:\\".to_string()
    } else if path.ends_with('\\') || path.ends_with('/') {
        path.replace('/', "\\")
    } else {
        format!("{}\\", path.replace('/', "\\"))
    };
    let wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

    // SAFETY: wide is a null-terminated UTF-16 string representing a valid drive path.
    // Out parameters are valid mutable references to u64 values.
    unsafe {
        let mut free_avail: u64 = 0;
        let mut total: u64 = 0;
        let mut free_total: u64 = 0;
        if crate::modules::win_util::ffi::GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_avail,
            &mut total,
            &mut free_total,
        ) != 0
        {
            if total == 0 {
                return None;
            }
            let used = total.saturating_sub(free_avail);
            let percentage = ((used as f64 / total as f64) * 100.0).round().min(100.0) as u8;
            return Some(DiskUsage {
                total_bytes: total,
                used_bytes: used,
                free_bytes: free_avail,
                percentage,
            });
        }
    }
    None
}

/// Formats disk usage into TiB, GiB, or MiB representation.
pub fn format_disk_usage(info: &DiskUsage) -> String {
    const TIB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;

    let total_f = info.total_bytes as f64;
    let used_f = info.used_bytes as f64;

    if total_f >= TIB {
        format!(
            "{:.2} TiB / {:.2} TiB ({}%)",
            used_f / TIB,
            total_f / TIB,
            info.percentage
        )
    } else if total_f >= GIB {
        format!(
            "{:.1} GiB / {:.1} GiB ({}%)",
            used_f / GIB,
            total_f / GIB,
            info.percentage
        )
    } else {
        format!(
            "{:.0} MiB / {:.0} MiB ({}%)",
            used_f / MIB,
            total_f / MIB,
            info.percentage
        )
    }
}

#[cfg(any(not(windows), test))]
const IGNORED_FS_TYPES: &[&str] = &[
    "tmpfs",
    "devtmpfs",
    "proc",
    "sysfs",
    "cgroup",
    "cgroup2",
    "overlay",
    "squashfs",
    "tracefs",
    "debugfs",
    "pstore",
    "bpf",
    "fusectl",
    "configfs",
    "binfmt_misc",
    "securityfs",
    "mqueue",
    "hugetlbfs",
    "autofs",
    "ramfs",
    "devpts",
    "nsfs",
    "efivarfs",
    "selinuxfs",
    "fuse.gvfsd-fuse",
    "fuse.portal",
    "erofs",
    "rootfs",
    "sdcardfs",
];

#[cfg(any(not(windows), test))]
const IGNORED_MOUNT_PREFIXES: &[&str] = &[
    "/mnt/wsl",
    "/mnt/wslg",
    "/usr/lib/wsl/drivers",
    "/init",
    "/dev",
    "/run",
    "/sys",
    "/proc",
    "/var/lib/docker",
    "/var/lib/containers",
    "/var/lib/flatpak",
    "/snap",
    "/apex",
    "/bootstrap-apex",
    "/data/app",
    "/data/user",
    "/data/data",
    "/data/media",
    "/data_mirror",
    "/storage/emulated",
    "/mnt/runtime",
    "/mnt/user",
    "/mnt/installer",
    "/mnt/androidwritable",
    "/mnt/pass_through",
    "/mnt/media_rw",
    "/system",
    "/system_ext",
    "/vendor",
    "/product",
    "/odm",
    "/oem",
    "/metadata",
    "/acct",
    "/config",
    "/linkerconfig",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionEntry {
    pub mount_point: String,
    pub display_name: String,
    pub fs_type: String,
    pub usage: DiskUsage,
}

/// Normalizes filesystem type names. Maps WSL virtual network protocols (9p, drvfs, 9pnet_virtio)
/// representing mounted Windows physical drives to "ntfs".
pub fn normalize_fs_type(fs_type: &str, _mount_point: &str) -> String {
    let lower = fs_type.to_lowercase();
    if lower == "9p" || lower == "drvfs" || lower == "9pnet_virtio" {
        "ntfs".to_string()
    } else {
        lower
    }
}

#[cfg(not(windows))]
pub fn get_fs_type_for_path(target: &str) -> Option<String> {
    let content = std::fs::read_to_string("/proc/mounts").ok()?;
    let mut best_match: Option<(&str, &str)> = None;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let mount = parts[1];
            let fs = parts[2];
            if target == mount
                || (target.starts_with(mount)
                    && (mount == "/" || target[mount.len()..].starts_with('/')))
            {
                if let Some((best_mount, _)) = best_match {
                    if mount.len() > best_mount.len() {
                        best_match = Some((mount, fs));
                    }
                } else {
                    best_match = Some((mount, fs));
                }
            }
        }
    }
    best_match.map(|(mount, fs)| normalize_fs_type(fs, mount))
}

#[cfg(windows)]
pub fn get_fs_type_for_path(target: &str) -> Option<String> {
    get_volume_fs_type(target)
}

#[cfg(windows)]
pub fn get_volume_fs_type(path: &str) -> Option<String> {
    use crate::modules::win_util::ffi;
    let root_path = if path.is_empty() || path == "/" {
        "C:\\".to_string()
    } else if path.ends_with('\\') || path.ends_with('/') {
        path.replace('/', "\\")
    } else {
        format!("{}\\", path.replace('/', "\\"))
    };
    let wide: Vec<u16> = root_path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut fs_name_buf = [0u16; 256];
    // SAFETY: wide is a null-terminated UTF-16 volume path. fs_name_buf has capacity of 256 u16 elements
    // which GetVolumeInformationW safely writes into up to fs_name_buf.len() elements.
    unsafe {
        if ffi::GetVolumeInformationW(
            wide.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            fs_name_buf.as_mut_ptr(),
            fs_name_buf.len() as u32,
        ) != 0
        {
            let len = fs_name_buf.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                return Some(String::from_utf16_lossy(&fs_name_buf[..len]));
            }
        }
    }
    None
}

/// Formats a complete disk display string including mount label, capacity, and filesystem type.
pub fn format_disk_entry(display_label: &str, usage: &DiskUsage, fs_type: Option<&str>) -> String {
    let base = format!("({}) {}", display_label, format_disk_usage(usage));
    if let Some(fs) = fs_type {
        if !fs.is_empty() {
            return format!("{} - {}", base, fs);
        }
    }
    base
}

/// Enumerates all real physical/virtual mount partitions from `/proc/mounts`.
/// Filters virtual kernel filesystems, snap/container overlays, and Android mount points.
#[cfg(not(windows))]
pub fn get_all_disks() -> Vec<PartitionEntry> {
    let mut entries = Vec::new();
    let Ok(content) = std::fs::read_to_string("/proc/mounts") else {
        if let Some(usage) = get_disk_usage("/") {
            entries.push(PartitionEntry {
                mount_point: "/".to_string(),
                display_name: "/".to_string(),
                fs_type: "ext4".to_string(),
                usage,
            });
        }
        return entries;
    };

    let mut seen_mounts = std::collections::HashSet::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let mount_point = parts[1];
        let fs_type = parts[2];

        // Ignore kernel pseudo-filesystems (proc, sysfs, tmpfs, cgroups)
        if IGNORED_FS_TYPES.contains(&fs_type) {
            continue;
        }

        // Ignore container overlays, flatpak sandboxes, and WSL internal virtual mounts
        if IGNORED_MOUNT_PREFIXES
            .iter()
            .any(|prefix| mount_point.starts_with(prefix))
        {
            continue;
        }

        if !seen_mounts.insert(mount_point.to_string()) {
            continue;
        }

        if let Some(usage) = get_disk_usage(mount_point) {
            // In WSL2, map 9P/drvfs drive mounts (/mnt/c -> C, /mnt/d -> D)
            let display_name = if let Some(wsl_drive) = mount_point.strip_prefix("/mnt/") {
                if wsl_drive.len() == 1 && wsl_drive.chars().next().unwrap().is_ascii_alphabetic() {
                    wsl_drive.to_uppercase()
                } else {
                    mount_point.to_string()
                }
            } else {
                mount_point.to_string()
            };

            let effective_fs = normalize_fs_type(fs_type, mount_point);

            entries.push(PartitionEntry {
                mount_point: mount_point.to_string(),
                display_name,
                fs_type: effective_fs,
                usage,
            });
        }
    }

    if entries.is_empty() {
        if let Some(usage) = get_disk_usage("/") {
            entries.push(PartitionEntry {
                mount_point: "/".to_string(),
                display_name: "/".to_string(),
                fs_type: "ext4".to_string(),
                usage,
            });
        }
    }

    // Always sort so root / comes first (Disk0), followed by other mounted partitions
    entries.sort_by(|a, b| {
        if a.mount_point == "/" {
            std::cmp::Ordering::Less
        } else if b.mount_point == "/" {
            std::cmp::Ordering::Greater
        } else {
            a.display_name.cmp(&b.display_name)
        }
    });

    entries
}

/// Enumerates all accessible logical drives on Windows.
#[cfg(windows)]
pub fn get_all_disks() -> Vec<PartitionEntry> {
    use crate::modules::win_util::ffi;
    let mut entries = Vec::new();
    // SAFETY: GetLogicalDrives is a simple Win32 query that retrieves a bitmask of valid drive letters.
    let drives_mask = unsafe { ffi::GetLogicalDrives() };

    for i in 0..26 {
        if (drives_mask & (1 << i)) != 0 {
            let drive_letter = (b'A' + i) as char;
            let root_path = format!("{}:\\", drive_letter);
            let wide: Vec<u16> = root_path.encode_utf16().chain(std::iter::once(0)).collect();
            // SAFETY: wide is a null-terminated UTF-16 root path string.
            let drive_type = unsafe { ffi::GetDriveTypeW(wide.as_ptr()) };
            // DRIVE_REMOVABLE = 2, DRIVE_FIXED = 3
            if drive_type == 2 || drive_type == 3 {
                if let Some(usage) = get_disk_usage(&root_path) {
                    let fs_type =
                        get_volume_fs_type(&root_path).unwrap_or_else(|| "NTFS".to_string());
                    entries.push(PartitionEntry {
                        mount_point: root_path,
                        display_name: format!("{}:", drive_letter),
                        fs_type,
                        usage,
                    });
                }
            }
        }
    }

    if entries.is_empty() {
        if let Some(usage) = get_disk_usage("C:\\") {
            let fs_type = get_volume_fs_type("C:\\").unwrap_or_else(|| "NTFS".to_string());
            entries.push(PartitionEntry {
                mount_point: "C:\\".to_string(),
                display_name: "C:".to_string(),
                fs_type,
                usage,
            });
        }
    }

    entries.sort_by(|a, b| {
        if a.display_name == "C:" {
            std::cmp::Ordering::Less
        } else if b.display_name == "C:" {
            std::cmp::Ordering::Greater
        } else {
            a.display_name.cmp(&b.display_name)
        }
    });

    entries
}

pub struct DiskCollector;

impl Collector for DiskCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Disk
    }

    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        let is_default_root = ctx.disk_target_path == "/";
        let target = if cfg!(windows) && is_default_root {
            "C:\\"
        } else {
            &ctx.disk_target_path
        };
        let usage = get_disk_usage(target)?;
        let display_label = if cfg!(windows) && is_default_root {
            "C:".to_string()
        } else {
            ctx.disk_target_path.clone()
        };
        let fs_type = get_fs_type_for_path(target);
        Some(ModuleOutput {
            id: ModuleId::Disk,
            label: "Disk0".to_string(),
            value: format_disk_entry(&display_label, &usage, fs_type.as_deref()),
            custom_rendered: None,
        })
    }

    fn collect_multiple(&self, ctx: &FetchContext) -> Vec<ModuleOutput> {
        let is_default =
            ctx.disk_target_path == "/" || (cfg!(windows) && ctx.disk_target_path == "C:\\");
        if !is_default {
            if let Some(usage) = get_disk_usage(&ctx.disk_target_path) {
                let fs_type = get_fs_type_for_path(&ctx.disk_target_path);
                return vec![ModuleOutput {
                    id: ModuleId::Disk,
                    label: "Disk0".to_string(),
                    value: format_disk_entry(&ctx.disk_target_path, &usage, fs_type.as_deref()),
                    custom_rendered: None,
                }];
            } else {
                return Vec::new();
            }
        }

        let disks = get_all_disks();
        let mut outputs = Vec::new();

        for (idx, entry) in disks.iter().enumerate() {
            let label = format!("Disk{}", idx);
            let value = format_disk_entry(&entry.display_name, &entry.usage, Some(&entry.fs_type));
            outputs.push(ModuleOutput {
                id: ModuleId::Disk,
                label,
                value,
                custom_rendered: None,
            });
        }

        outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_disk_usage_gib() {
        let usage = DiskUsage {
            total_bytes: 250 * 1024 * 1024 * 1024,
            used_bytes: 32 * 1024 * 1024 * 1024,
            free_bytes: 218 * 1024 * 1024 * 1024,
            percentage: 13,
        };
        let s = format_disk_usage(&usage);
        assert_eq!(s, "32.0 GiB / 250.0 GiB (13%)");
    }

    #[test]
    fn test_format_disk_usage_tib() {
        let usage = DiskUsage {
            total_bytes: 2 * 1024 * 1024 * 1024 * 1024,
            used_bytes: 1024 * 1024 * 1024 * 1024,
            free_bytes: 1024 * 1024 * 1024 * 1024,
            percentage: 50,
        };
        let s = format_disk_usage(&usage);
        assert_eq!(s, "1.00 TiB / 2.00 TiB (50%)");
    }

    #[test]
    fn test_format_disk_usage_mib() {
        let usage = DiskUsage {
            total_bytes: 500 * 1024 * 1024,
            used_bytes: 100 * 1024 * 1024,
            free_bytes: 400 * 1024 * 1024,
            percentage: 20,
        };
        let s = format_disk_usage(&usage);
        assert_eq!(s, "100 MiB / 500 MiB (20%)");
    }

    #[test]
    fn test_get_disk_usage_root() {
        let usage = get_disk_usage("/");
        assert!(usage.is_some());
        let u = usage.unwrap();
        assert!(u.total_bytes > 0);
    }

    #[test]
    fn test_get_disk_usage_invalid_paths() {
        assert_eq!(get_disk_usage("/nonexistent_path_xyz_987654"), None);
        assert_eq!(get_disk_usage("invalid\0nullbyte"), None);
    }

    #[test]
    fn test_android_mounts_filter() {
        let sample_mounts = r#"
/dev/root / ext4 ro,relatime 0 0
/dev/block/dm-0 /apex/com.android.runtime erofs ro,relatime 0 0
/dev/block/dm-1 /apex/com.android.art@361099999 erofs ro,relatime 0 0
/dev/block/dm-2 /bootstrap-apex/com.android.runtime erofs ro,relatime 0 0
/dev/block/bootdevice/by-name/userdata /data f2fs rw,nosuid,nodev,noatime 0 0
/dev/block/dm-3 /data/app/~~XYZ==/com.google.android.youtube==/base.apk erofs ro,nodev 0 0
/data/media /storage/emulated sdcardfs rw,nosuid,nodev 0 0
/dev/block/vold/public:179,1 /storage/FF70-CD48 vfat rw,dirsync,nosuid,nodev 0 0
/dev/block/bootdevice/by-name/product /product erofs ro,relatime 0 0
/dev/block/bootdevice/by-name/vendor /vendor erofs ro,relatime 0 0
/dev/block/bootdevice/by-name/metadata /metadata f2fs rw,sync 0 0
"#;

        let filtered: Vec<&str> = sample_mounts
            .lines()
            .filter_map(|l| {
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() < 3 {
                    return None;
                }
                let mp = parts[1];
                let fs = parts[2];
                if IGNORED_FS_TYPES.contains(&fs) {
                    return None;
                }
                if IGNORED_MOUNT_PREFIXES
                    .iter()
                    .any(|prefix| mp.starts_with(prefix))
                {
                    return None;
                }
                Some(mp)
            })
            .collect();

        assert_eq!(filtered, vec!["/", "/data", "/storage/FF70-CD48"]);
    }

    #[test]
    fn test_format_disk_entry_with_fs() {
        let usage = DiskUsage {
            total_bytes: 500 * 1024 * 1024 * 1024,
            used_bytes: 100 * 1024 * 1024 * 1024,
            free_bytes: 400 * 1024 * 1024 * 1024,
            percentage: 20,
        };
        let formatted = format_disk_entry("/", &usage, Some("ext4"));
        assert_eq!(formatted, "(/) 100.0 GiB / 500.0 GiB (20%) - ext4");

        let formatted_no_fs = format_disk_entry("/", &usage, None);
        assert_eq!(formatted_no_fs, "(/) 100.0 GiB / 500.0 GiB (20%)");
    }

    #[test]
    fn test_normalize_fs_type() {
        assert_eq!(normalize_fs_type("9p", "/mnt/c"), "ntfs");
        assert_eq!(normalize_fs_type("drvfs", "/mnt/d"), "ntfs");
        assert_eq!(normalize_fs_type("9pnet_virtio", "/mnt/c"), "ntfs");
        assert_eq!(normalize_fs_type("ext4", "/"), "ext4");
        assert_eq!(normalize_fs_type("btrfs", "/home"), "btrfs");
        assert_eq!(normalize_fs_type("zfs", "/pool"), "zfs");
    }
}
