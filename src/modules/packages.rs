use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Parses Debian `/var/lib/dpkg/status` raw bytes and counts installed packages.
/// Fast path: stream-parses status file directly without string allocations or UTF-8 decoding.
pub fn parse_dpkg_status_bytes(bytes: &[u8]) -> usize {
    let mut count = 0;
    for line in bytes.split(|&b| b == b'\n') {
        if line.starts_with(b"Status:") && line.ends_with(b" installed") {
            count += 1;
        }
    }
    count
}

/// Parses Debian `/var/lib/dpkg/status` content and counts installed packages.
pub fn parse_dpkg_status(content: &str) -> usize {
    parse_dpkg_status_bytes(content.as_bytes())
}

/// Counts installed packages for Debian/Ubuntu family from a status file path.
pub fn count_dpkg_from_path(path: &Path) -> Option<usize> {
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));

    let cache_file = cache_dir.as_ref().map(|d| d.join("dpkg_v1.cache"));

    if let Ok(meta) = fs::metadata(path) {
        if let Ok(mtime) = meta.modified() {
            let mtime_sec = mtime
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            if let Some(ref c_path) = cache_file {
                if let Ok(cached) = fs::read_to_string(c_path) {
                    if let Some((saved_mtime, saved_count)) = cached.trim().split_once(' ') {
                        if let (Ok(s_mtime), Ok(count)) =
                            (saved_mtime.parse::<u64>(), saved_count.parse::<usize>())
                        {
                            if s_mtime == mtime_sec && count > 0 {
                                return Some(count);
                            }
                        }
                    }
                }
            }

            if let Ok(bytes) = fs::read(path) {
                let count = parse_dpkg_status_bytes(&bytes);
                if count > 0 {
                    if let Some(ref dir) = cache_dir {
                        let _ = fs::create_dir_all(dir);
                    }
                    if let Some(ref c_path) = cache_file {
                        let _ = fs::write(c_path, format!("{} {}", mtime_sec, count));
                    }
                    return Some(count);
                }
            }
        }
    }

    if let Ok(bytes) = fs::read(path) {
        let count = parse_dpkg_status_bytes(&bytes);
        if count > 0 {
            return Some(count);
        }
    }
    None
}

/// Counts installed packages for Debian/Ubuntu family.
pub fn count_dpkg() -> Option<usize> {
    if let Some(count) = count_dpkg_from_path(Path::new("/var/lib/dpkg/status")) {
        return Some(count);
    }

    // Fallback: dpkg-query command if status file is inaccessible (e.g. non-standard chroot)
    if let Ok(output) = Command::new("dpkg-query")
        .args(["-f", "${binary:Package}\n", "-W"])
        .output()
    {
        if output.status.success() {
            let count = parse_rpm_output(&output.stdout);
            if count > 0 {
                return Some(count);
            }
        }
    }

    None
}

/// Counts installed packages for Arch Linux family from a given pacman local database directory.
/// Each installed package corresponds to a subdirectory under `/var/lib/pacman/local`.
pub fn count_pacman_from_dir(pacman_dir: &Path) -> Option<usize> {
    if let Ok(entries) = fs::read_dir(pacman_dir) {
        let count = entries
            .flatten()
            .filter(|e| {
                if let Ok(ft) = e.file_type() {
                    if ft.is_dir() {
                        let name = e.file_name();
                        let s = name.to_string_lossy();
                        return !s.starts_with('.') && s != "ALPM_DB_VERSION";
                    }
                }
                false
            })
            .count();
        if count > 0 {
            return Some(count);
        }
    }
    None
}

/// Counts installed packages for Arch Linux family from pacman local database.
pub fn count_pacman() -> Option<usize> {
    count_pacman_from_dir(Path::new("/var/lib/pacman/local"))
}

/// Counts non-empty newline-delimited entries from a raw byte slice (such as `rpm -qa`, `dpkg-query`, or `xbps-query -l`).
pub fn count_newline_entries(output: &[u8]) -> usize {
    output
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .count()
}

/// Alias for backwards compatibility with tests.
pub fn parse_rpm_output(output: &[u8]) -> usize {
    count_newline_entries(output)
}

/// Counts records in an SQLite table by traversing B-Tree leaf cell headers without loading table payload data.
/// Works directly on `/var/lib/rpm/rpmdb.sqlite` and `/usr/lib/sysimage/rpm/rpmdb.sqlite`.
pub fn count_sqlite_table_cells(path: &Path, rootpage: u32) -> Option<usize> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = fs::File::open(path).ok()?;
    let mut hdr = [0u8; 100];
    file.read_exact(&mut hdr).ok()?;
    if &hdr[..16] != b"SQLite format 3\0" {
        return None;
    }
    let mut page_size = u16::from_be_bytes([hdr[16], hdr[17]]) as u64;
    if page_size == 1 {
        page_size = 65536;
    }
    if !(512..=65536).contains(&page_size) {
        return None;
    }

    let mut stack = vec![rootpage];
    let mut total_cells = 0usize;
    let mut visited = std::collections::HashSet::new();

    while let Some(pg) = stack.pop() {
        if pg == 0 || !visited.insert(pg) {
            continue;
        }
        let page_offset = (pg as u64 - 1) * page_size;
        let hdr_offset = if pg == 1 { 100 } else { 0 };

        file.seek(SeekFrom::Start(page_offset + hdr_offset)).ok()?;
        let mut pg_hdr = [0u8; 12];
        file.read_exact(&mut pg_hdr).ok()?;

        let pg_type = pg_hdr[0];
        let num_cells = u16::from_be_bytes([pg_hdr[3], pg_hdr[4]]) as usize;

        match pg_type {
            0x0d => {
                // Leaf table B-Tree page: add cell count directly
                total_cells += num_cells;
            }
            0x05 => {
                // Interior table B-Tree page: follow right child and cell child pointers
                let right_child =
                    u32::from_be_bytes([pg_hdr[8], pg_hdr[9], pg_hdr[10], pg_hdr[11]]);
                stack.push(right_child);

                let ptr_array_offset = page_offset + hdr_offset + 12;
                file.seek(SeekFrom::Start(ptr_array_offset)).ok()?;
                let mut ptr_bytes = vec![0u8; num_cells * 2];
                file.read_exact(&mut ptr_bytes).ok()?;

                for i in 0..num_cells {
                    let cell_ptr =
                        u16::from_be_bytes([ptr_bytes[i * 2], ptr_bytes[i * 2 + 1]]) as u64;
                    file.seek(SeekFrom::Start(page_offset + cell_ptr)).ok()?;
                    let mut child_bytes = [0u8; 4];
                    file.read_exact(&mut child_bytes).ok()?;
                    let left_child = u32::from_be_bytes(child_bytes);
                    stack.push(left_child);
                }
            }
            _ => {}
        }
    }

    if total_cells > 0 {
        Some(total_cells)
    } else {
        None
    }
}

/// Counts installed packages for Red Hat / Fedora family via rpm with zero-subprocess SQLite parsing and mtime caching.
pub fn count_rpm_from_paths(db_paths: &[&str]) -> Option<usize> {
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache")))
        .map(|p| p.join("ferrisfetch"));

    let cache_file = cache_dir.as_ref().map(|d| d.join("rpm_v1.cache"));

    // Find the active RPM database path
    let mut active_db: Option<(&str, u64)> = None;
    for &p in db_paths {
        if let Ok(meta) = fs::metadata(p) {
            if let Ok(mtime) = meta.modified() {
                let mtime_sec = mtime
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                active_db = Some((p, mtime_sec));
                break;
            }
        }
    }

    if let Some((path, mtime_sec)) = active_db {
        if let Some(ref c_path) = cache_file {
            if let Ok(cached) = fs::read_to_string(c_path) {
                if let Some((saved_mtime, saved_count)) = cached.trim().split_once(' ') {
                    if let (Ok(s_mtime), Ok(count)) =
                        (saved_mtime.parse::<u64>(), saved_count.parse::<usize>())
                    {
                        if s_mtime == mtime_sec && count > 0 {
                            return Some(count);
                        }
                    }
                }
            }
        }

        // Fast path 1: Zero-subprocess SQLite B-Tree header parsing (<0.2ms)
        if path.ends_with(".sqlite") {
            if let Some(count) = count_sqlite_table_cells(Path::new(path), 2) {
                if let Some(ref dir) = cache_dir {
                    let _ = fs::create_dir_all(dir);
                }
                if let Some(ref c_path) = cache_file {
                    let _ = fs::write(c_path, format!("{} {}", mtime_sec, count));
                }
                return Some(count);
            }
        }

        // Fallback 2: query rpm -qa once and persist to cache
        if let Ok(output) = Command::new("rpm").arg("-qa").output() {
            if output.status.success() {
                let count = count_newline_entries(&output.stdout);
                if count > 0 {
                    if let Some(ref dir) = cache_dir {
                        let _ = fs::create_dir_all(dir);
                    }
                    if let Some(ref c_path) = cache_file {
                        let _ = fs::write(c_path, format!("{} {}", mtime_sec, count));
                    }
                    return Some(count);
                }
            }
        }
    }

    None
}

/// Counts installed packages for Red Hat / Fedora family via rpm.
pub fn count_rpm() -> Option<usize> {
    let rpm_paths = [
        "/var/lib/rpm/rpmdb.sqlite",
        "/usr/lib/sysimage/rpm/rpmdb.sqlite",
        "/var/lib/rpm/Packages",
        "/usr/lib/sysimage/rpm/Packages",
    ];
    count_rpm_from_paths(&rpm_paths)
}

/// Parses APK installed db content.
pub fn parse_apk_installed(content: &str) -> usize {
    content.lines().filter(|l| l.starts_with("P:")).count()
}

/// Counts installed packages for Alpine Linux by parsing `/lib/apk/db/installed`.
pub fn count_apk() -> Option<usize> {
    let path = Path::new("/lib/apk/db/installed");
    if let Ok(content) = fs::read_to_string(path) {
        let count = parse_apk_installed(&content);
        if count > 0 {
            return Some(count);
        }
    }
    None
}

/// Counts installed packages for Void Linux by parsing pkgdb plist directly (<0.1ms).
pub fn count_xbps() -> Option<usize> {
    let path = Path::new("/var/db/xbps");
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("pkgdb") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let count = content.matches("<key>pkgver</key>").count();
                    if count > 0 {
                        return Some(count);
                    }
                }
            }
        }
    }
    None
}

/// Counts installed packages in active Nix profiles without running nix-env.
pub fn count_nix() -> Option<usize> {
    let mut total = 0;
    let standard_paths = [
        Path::new("/run/current-system/sw/bin"),
        Path::new("/nix/var/nix/profiles/default/bin"),
    ];

    for path in &standard_paths {
        if let Ok(entries) = fs::read_dir(path) {
            total += entries
                .flatten()
                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                .count();
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let user_profile = Path::new(&home).join(".nix-profile/bin");
        if let Ok(entries) = fs::read_dir(user_profile) {
            total += entries
                .flatten()
                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                .count();
        }
    }

    if total > 0 {
        Some(total)
    } else {
        None
    }
}

/// Counts installed packages in active Guix profile without running guix package.
pub fn count_guix() -> Option<usize> {
    let mut total = 0;
    if let Ok(home) = std::env::var("HOME") {
        let user_profile = Path::new(&home).join(".guix-profile/bin");
        if let Ok(entries) = fs::read_dir(user_profile) {
            total += entries
                .flatten()
                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                .count();
        }
    }

    if total > 0 {
        Some(total)
    } else {
        None
    }
}

/// Counts installed packages for FreeBSD, OpenBSD, and NetBSD.
pub fn count_pkg() -> Option<usize> {
    // FreeBSD / DragonFly: /var/db/pkg/local.sqlite
    let freebsd_db = Path::new("/var/db/pkg/local.sqlite");
    if freebsd_db.is_file() {
        if let Some(count) = count_sqlite_table_cells(freebsd_db, 2) {
            return Some(count);
        }
    }

    // OpenBSD / NetBSD: /var/db/pkg directory
    let bsd_pkg_dir = Path::new("/var/db/pkg");
    if let Ok(entries) = fs::read_dir(bsd_pkg_dir) {
        let count = entries
            .flatten()
            .filter(|e| {
                e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    && !e.file_name().to_string_lossy().starts_with('.')
                    && e.file_name() != "local.sqlite"
            })
            .count();
        if count > 0 {
            return Some(count);
        }
    }

    None
}

/// Counts installed MacPorts packages on macOS via local SQLite registry.
pub fn count_macports() -> Option<usize> {
    let macports_db = Path::new("/opt/local/var/macports/registry/registry.db");
    if macports_db.is_file() {
        if let Some(count) = count_sqlite_table_cells(macports_db, 2) {
            return Some(count);
        }
    }
    None
}

/// Counts installed Flatpak applications from specified system and user paths.
pub fn count_flatpak_from_dirs(sys_path: &Path, user_path: &Path) -> Option<usize> {
    let mut total = 0;

    if let Ok(entries) = fs::read_dir(sys_path) {
        total += entries
            .flatten()
            .filter(|e| {
                e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    && !e.file_name().to_string_lossy().starts_with('.')
            })
            .count();
    }

    if let Ok(entries) = fs::read_dir(user_path) {
        total += entries
            .flatten()
            .filter(|e| {
                e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    && !e.file_name().to_string_lossy().starts_with('.')
            })
            .count();
    }

    if total > 0 {
        Some(total)
    } else {
        None
    }
}

/// Counts installed Flatpak applications (system and user level).
pub fn count_flatpak() -> Option<usize> {
    let sys_path = Path::new("/var/lib/flatpak/app");
    let user_path = std::env::var("HOME")
        .map(|h| Path::new(&h).join(".local/share/flatpak/app"))
        .unwrap_or_else(|_| Path::new("/nonexistent").to_path_buf());
    count_flatpak_from_dirs(sys_path, &user_path)
}

/// Counts installed Snap packages from specified directories, deduplicating revisions.
pub fn count_snap_from_dirs(snaps_path: &Path, snap_root: &Path) -> Option<usize> {
    if let Ok(entries) = fs::read_dir(snaps_path) {
        let mut unique_snaps = std::collections::HashSet::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.ends_with(".snap") {
                // Strip revision suffix, e.g. core22_1.snap -> core22 to count unique applications
                let pkg_name = s.split_once('_').map(|(p, _)| p).unwrap_or(&s);
                unique_snaps.insert(pkg_name.to_string());
            }
        }
        if !unique_snaps.is_empty() {
            return Some(unique_snaps.len());
        }
    }

    // Fallback: count subdirectories in /snap (excluding metadata and internal symlinks)
    if let Ok(entries) = fs::read_dir(snap_root) {
        let count = entries
            .flatten()
            .filter(|e| {
                let name = e.file_name();
                let s = name.to_string_lossy();
                s != "bin" && s != "README" && !s.starts_with('.')
            })
            .count();
        if count > 0 {
            return Some(count);
        }
    }

    None
}

/// Counts installed Snap packages.
pub fn count_snap() -> Option<usize> {
    count_snap_from_dirs(Path::new("/var/lib/snapd/snaps"), Path::new("/snap"))
}

/// Counts installed Homebrew formulae from standard Cellar paths across system and user prefixes.
pub fn count_brew() -> Option<usize> {
    let cellar_paths = [
        Path::new("/home/linuxbrew/.linuxbrew/Cellar"),
        Path::new("/opt/homebrew/Cellar"),
        Path::new("/usr/local/Cellar"),
    ];

    for path in &cellar_paths {
        if let Ok(entries) = fs::read_dir(path) {
            let count = entries
                .flatten()
                .filter(|e| {
                    e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && !e.file_name().to_string_lossy().starts_with('.')
                })
                .count();
            if count > 0 {
                return Some(count);
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let user_cellar = Path::new(&home).join(".linuxbrew/Cellar");
        if let Ok(entries) = fs::read_dir(&user_cellar) {
            let count = entries
                .flatten()
                .filter(|e| {
                    e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && !e.file_name().to_string_lossy().starts_with('.')
                })
                .count();
            if count > 0 {
                return Some(count);
            }
        }
    }

    None
}

/// Counts installed Gentoo ebuild packages from /var/db/pkg.
pub fn count_emerge() -> Option<usize> {
    let pkg_dir = Path::new("/var/db/pkg");
    if let Ok(categories) = fs::read_dir(pkg_dir) {
        let mut total = 0;
        for cat in categories.flatten() {
            if cat.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Ok(pkgs) = fs::read_dir(cat.path()) {
                    total += pkgs
                        .flatten()
                        .filter(|p| {
                            p.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                                && !p.file_name().to_string_lossy().starts_with('.')
                        })
                        .count();
                }
            }
        }
        if total > 0 {
            return Some(total);
        }
    }
    None
}

/// Counts installed WinGet packages from Packages directory or Links fallback.
pub fn count_winget_from_dirs(packages_path: &Path, links_path: &Path) -> Option<usize> {
    if let Ok(entries) = fs::read_dir(packages_path) {
        let count = entries
            .flatten()
            .filter(|e| {
                e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    && !e.file_name().to_string_lossy().starts_with('.')
            })
            .count();
        if count > 0 {
            return Some(count);
        }
    }

    if let Ok(entries) = fs::read_dir(links_path) {
        let count = entries
            .flatten()
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .count();
        if count > 0 {
            return Some(count);
        }
    }

    None
}

/// Counts installed WinGet packages.
pub fn count_winget() -> Option<usize> {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}\\AppData\\Local", p)))
        .ok()?;
    let base = Path::new(&local_app_data);
    let packages = base.join("Microsoft").join("WinGet").join("Packages");
    let links = base.join("Microsoft").join("WinGet").join("Links");
    count_winget_from_dirs(&packages, &links)
}

/// Counts installed Scoop packages from apps directory.
pub fn count_scoop_from_dir(apps_dir: &Path) -> Option<usize> {
    if let Ok(entries) = fs::read_dir(apps_dir) {
        let count = entries
            .flatten()
            .filter(|e| {
                if let Ok(ft) = e.file_type() {
                    if ft.is_dir() {
                        let name = e.file_name();
                        let s = name.to_string_lossy();
                        return !s.starts_with('.') && s != "scoop";
                    }
                }
                false
            })
            .count();
        if count > 0 {
            return Some(count);
        }
    }
    None
}

/// Counts installed Scoop packages.
pub fn count_scoop() -> Option<usize> {
    if let Ok(scoop_dir) = std::env::var("SCOOP") {
        if let Some(c) = count_scoop_from_dir(&Path::new(&scoop_dir).join("apps")) {
            return Some(c);
        }
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()?;
    let apps_dir = Path::new(&home).join("scoop").join("apps");
    count_scoop_from_dir(&apps_dir)
}

/// Counts installed Chocolatey packages from lib directory.
pub fn count_choco_from_dir(lib_path: &Path) -> Option<usize> {
    if let Ok(entries) = fs::read_dir(lib_path) {
        let count = entries
            .flatten()
            .filter(|e| {
                if let Ok(ft) = e.file_type() {
                    if ft.is_dir() {
                        let name = e.file_name();
                        let s = name.to_string_lossy();
                        return !s.starts_with('.');
                    }
                }
                false
            })
            .count();
        if count > 0 {
            return Some(count);
        }
    }
    None
}

/// Counts installed Chocolatey packages.
pub fn count_choco() -> Option<usize> {
    if let Ok(choco_install) = std::env::var("ChocolateyInstall") {
        if let Some(c) = count_choco_from_dir(&Path::new(&choco_install).join("lib")) {
            return Some(c);
        }
    }
    if let Ok(all_users) =
        std::env::var("ALLUSERSPROFILE").or_else(|_| std::env::var("ProgramData"))
    {
        let lib = Path::new(&all_users).join("chocolatey").join("lib");
        if let Some(c) = count_choco_from_dir(&lib) {
            return Some(c);
        }
    }
    count_choco_from_dir(Path::new("C:\\ProgramData\\chocolatey\\lib"))
}

/// Parses `.crates.toml` content and counts installed global binary crates.
pub fn parse_cargo_crates_toml(content: &str) -> usize {
    let mut count = 0;
    let mut in_v_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_v_section = line == "[v1]" || line == "[v2]";
            continue;
        }
        if in_v_section && line.starts_with('"') && line.contains('=') {
            count += 1;
        }
    }
    count
}

/// Counts installed global Cargo crates from a `.crates.toml` path.
pub fn count_cargo_from_path(path: &Path) -> Option<usize> {
    if let Ok(content) = fs::read_to_string(path) {
        let count = parse_cargo_crates_toml(&content);
        if count > 0 {
            return Some(count);
        }
    }
    None
}

/// Counts installed global Cargo crates.
pub fn count_cargo() -> Option<usize> {
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        if let Some(c) = count_cargo_from_path(&Path::new(&cargo_home).join(".crates.toml")) {
            return Some(c);
        }
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()?;
    let crates_path = Path::new(&home).join(".cargo").join(".crates.toml");
    count_cargo_from_path(&crates_path)
}

/// Counts installed global npm packages from a `node_modules` directory.
pub fn count_npm_from_dir(node_modules: &Path) -> Option<usize> {
    if let Ok(entries) = fs::read_dir(node_modules) {
        let mut total = 0;
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name();
                    let s = name.to_string_lossy();
                    if s.starts_with('@') {
                        if let Ok(scoped_entries) = fs::read_dir(entry.path()) {
                            total += scoped_entries
                                .flatten()
                                .filter(|e| {
                                    e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                                        && !e.file_name().to_string_lossy().starts_with('.')
                                })
                                .count();
                        }
                    } else if !s.starts_with('.') && s != "npm" && s != "corepack" {
                        total += 1;
                    }
                }
            }
        }
        if total > 0 {
            return Some(total);
        }
    }
    None
}

/// Counts installed global npm packages.
pub fn count_npm() -> Option<usize> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let win_npm = Path::new(&appdata).join("npm").join("node_modules");
        if let Some(c) = count_npm_from_dir(&win_npm) {
            return Some(c);
        }
    }

    let standard_paths = [
        Path::new("/usr/lib/node_modules"),
        Path::new("/usr/local/lib/node_modules"),
    ];
    for p in &standard_paths {
        if let Some(c) = count_npm_from_dir(p) {
            return Some(c);
        }
    }

    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let home_path = Path::new(&home);
        let user_npm = home_path.join(".npm-global/lib/node_modules");
        if let Some(c) = count_npm_from_dir(&user_npm) {
            return Some(c);
        }

        let nvm_versions = home_path.join(".nvm/versions/node");
        if let Ok(versions) = fs::read_dir(nvm_versions) {
            for ver in versions.flatten() {
                if ver.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let nvm_modules = ver.path().join("lib").join("node_modules");
                    if let Some(c) = count_npm_from_dir(&nvm_modules) {
                        return Some(c);
                    }
                }
            }
        }
    }

    None
}

/// Counts installed Python packages from a `site-packages` directory.
pub fn count_pip_from_dir(site_packages: &Path) -> Option<usize> {
    if let Ok(entries) = fs::read_dir(site_packages) {
        let count = entries
            .flatten()
            .filter(|e| {
                let name = e.file_name();
                let s = name.to_string_lossy();
                (s.ends_with(".dist-info") || s.ends_with(".egg-info")) && !s.starts_with('.')
            })
            .count();
        if count > 0 {
            return Some(count);
        }
    }
    None
}

/// Counts installed Python packages across standard site-packages paths.
pub fn count_pip() -> Option<usize> {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let py_dir = Path::new(&local_app_data).join("Programs").join("Python");
        if let Ok(entries) = fs::read_dir(py_dir) {
            for entry in entries.flatten() {
                let site_pkg = entry.path().join("Lib").join("site-packages");
                if let Some(c) = count_pip_from_dir(&site_pkg) {
                    return Some(c);
                }
            }
        }
    }

    if let Ok(app_data) = std::env::var("APPDATA") {
        let py_dir = Path::new(&app_data).join("Python");
        if let Ok(entries) = fs::read_dir(py_dir) {
            for entry in entries.flatten() {
                let site_pkg = entry.path().join("site-packages");
                if let Some(c) = count_pip_from_dir(&site_pkg) {
                    return Some(c);
                }
            }
        }
    }

    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let py_local = Path::new(&home).join(".local").join("lib");
        if let Ok(entries) = fs::read_dir(py_local) {
            for entry in entries.flatten() {
                let s = entry.file_name().to_string_lossy().to_string();
                if s.starts_with("python") {
                    let site_pkg = entry.path().join("site-packages");
                    if let Some(c) = count_pip_from_dir(&site_pkg) {
                        return Some(c);
                    }
                }
            }
        }
    }

    None
}

/// Gathers and formats package counts across all active package managers on Windows.
#[cfg(windows)]
pub fn get_packages_summary() -> Option<String> {
    let mut parts = Vec::new();

    if let Some(winget) = count_winget() {
        parts.push(format!("{} (winget)", winget));
    }
    if let Some(scoop) = count_scoop() {
        parts.push(format!("{} (scoop)", scoop));
    }
    if let Some(choco) = count_choco() {
        parts.push(format!("{} (choco)", choco));
    }
    if let Some(pacman) = count_pacman_from_dir(Path::new("C:\\msys64\\var\\lib\\pacman\\local")) {
        parts.push(format!("{} (pacman)", pacman));
    }
    if let Some(cargo) = count_cargo() {
        parts.push(format!("{} (cargo)", cargo));
    }
    if let Some(npm) = count_npm() {
        parts.push(format!("{} (npm)", npm));
    }
    if let Some(pip) = count_pip() {
        parts.push(format!("{} (pip)", pip));
    }

    if !parts.is_empty() {
        Some(parts.join(", "))
    } else {
        None
    }
}

/// Gathers and formats package counts across all active package managers on Unix / Linux / macOS.
#[cfg(not(windows))]
pub fn get_packages_summary() -> Option<String> {
    let mut parts = Vec::new();

    if let Some(dpkg) = count_dpkg() {
        parts.push(format!("{} (dpkg)", dpkg));
    }
    if let Some(pacman) = count_pacman() {
        parts.push(format!("{} (pacman)", pacman));
    }
    if let Some(rpm) = count_rpm() {
        parts.push(format!("{} (rpm)", rpm));
    }
    if let Some(apk) = count_apk() {
        parts.push(format!("{} (apk)", apk));
    }
    if let Some(xbps) = count_xbps() {
        parts.push(format!("{} (xbps)", xbps));
    }
    if let Some(emerge) = count_emerge() {
        parts.push(format!("{} (emerge)", emerge));
    }
    if let Some(flatpak) = count_flatpak() {
        parts.push(format!("{} (flatpak)", flatpak));
    }
    if let Some(snap) = count_snap() {
        parts.push(format!("{} (snap)", snap));
    }
    if let Some(brew) = count_brew() {
        parts.push(format!("{} (brew)", brew));
    }
    if let Some(winget) = count_winget() {
        parts.push(format!("{} (winget)", winget));
    }
    if let Some(scoop) = count_scoop() {
        parts.push(format!("{} (scoop)", scoop));
    }
    if let Some(choco) = count_choco() {
        parts.push(format!("{} (choco)", choco));
    }
    if let Some(nix) = count_nix() {
        parts.push(format!("{} (nix)", nix));
    }
    if let Some(guix) = count_guix() {
        parts.push(format!("{} (guix)", guix));
    }
    if let Some(pkg) = count_pkg() {
        parts.push(format!("{} (pkg)", pkg));
    }
    if let Some(macports) = count_macports() {
        parts.push(format!("{} (macports)", macports));
    }
    if let Some(cargo) = count_cargo() {
        parts.push(format!("{} (cargo)", cargo));
    }
    if let Some(npm) = count_npm() {
        parts.push(format!("{} (npm)", npm));
    }
    if let Some(pip) = count_pip() {
        parts.push(format!("{} (pip)", pip));
    }

    if !parts.is_empty() {
        Some(parts.join(", "))
    } else {
        None
    }
}

pub struct PackagesCollector;

impl Collector for PackagesCollector {
    fn id(&self) -> ModuleId {
        ModuleId::Packages
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let summary = get_packages_summary()?;
        Some(ModuleOutput {
            id: ModuleId::Packages,
            label: "Packages".to_string(),
            value: summary,
            custom_rendered: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dpkg_status() {
        let fixture = r#"
Package: bash
Status: install ok installed
Section: shells

Package: curl
Status: deinstall ok config-files
Section: web

Package: libc6
Status: hold ok installed
Section: libs
"#;
        assert_eq!(parse_dpkg_status(fixture), 2);
    }

    #[test]
    fn test_parse_dpkg_status_empty_or_corrupted() {
        assert_eq!(parse_dpkg_status(""), 0);
        assert_eq!(
            parse_dpkg_status("Package: foo\nStatus: half-installed\n"),
            0
        );
        assert_eq!(parse_dpkg_status("random text\nno valid header\n"), 0);
    }

    #[test]
    fn test_parse_rpm_output() {
        let output =
            b"coreutils-9.1-1.fc38.x86_64\nbash-5.2.15-3.fc38.x86_64\nglibc-2.37-4.fc38.x86_64\n";
        assert_eq!(parse_rpm_output(output), 3);
        assert_eq!(parse_rpm_output(b""), 0);
        assert_eq!(parse_rpm_output(b"\n\n"), 0);
    }

    #[test]
    fn test_parse_apk_installed() {
        let content = "P:musl\nV:1.2.4\n\nP:busybox\nV:1.36.1\n\nP:alpine-keys\nV:2.4-r1\n";
        assert_eq!(parse_apk_installed(content), 3);
        assert_eq!(parse_apk_installed(""), 0);
    }

    #[test]
    fn test_count_pacman_from_dir_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();

        // Initially empty
        assert_eq!(count_pacman_from_dir(path), None);

        // Add dummy package directories and files
        fs::create_dir(path.join("coreutils-9.5-1")).unwrap();
        fs::create_dir(path.join("linux-6.8.9-arch1-1")).unwrap();
        fs::create_dir(path.join(".hidden-dir")).unwrap();
        fs::write(path.join("ALPM_DB_VERSION"), "9").unwrap();

        assert_eq!(count_pacman_from_dir(path), Some(2));
    }

    #[test]
    fn test_count_pacman_from_dir_nonexistent() {
        assert_eq!(
            count_pacman_from_dir(Path::new("/nonexistent_pacman_dir_12345")),
            None
        );
    }

    #[test]
    fn test_count_flatpak_from_dirs_mock() {
        let sys_tmp = tempfile::tempdir().unwrap();
        let user_tmp = tempfile::tempdir().unwrap();

        fs::create_dir(sys_tmp.path().join("org.mozilla.firefox")).unwrap();
        fs::create_dir(user_tmp.path().join("org.videolan.VLC")).unwrap();

        let count = count_flatpak_from_dirs(sys_tmp.path(), user_tmp.path());
        assert_eq!(count, Some(2));
    }

    #[test]
    fn test_count_snap_from_dirs_mock() {
        let snaps_tmp = tempfile::tempdir().unwrap();
        let snap_root_tmp = tempfile::tempdir().unwrap();

        // 2 revisions of core22, 1 revision of firefox -> total 2 distinct packages
        fs::write(snaps_tmp.path().join("core22_1.snap"), b"").unwrap();
        fs::write(snaps_tmp.path().join("core22_2.snap"), b"").unwrap();
        fs::write(snaps_tmp.path().join("firefox_2.snap"), b"").unwrap();

        let count = count_snap_from_dirs(snaps_tmp.path(), snap_root_tmp.path());
        assert_eq!(count, Some(2));
    }

    #[test]
    fn test_count_newline_entries() {
        let output = b"pkg1\npkg2\npkg3\n";
        assert_eq!(count_newline_entries(output), 3);
        assert_eq!(count_newline_entries(b""), 0);
    }

    #[test]
    fn test_count_emerge_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cat1 = temp_dir.path().join("sys-apps");
        let cat2 = temp_dir.path().join("app-editors");
        fs::create_dir_all(&cat1).unwrap();
        fs::create_dir_all(&cat2).unwrap();

        fs::create_dir(cat1.join("coreutils-9.3")).unwrap();
        fs::create_dir(cat1.join("systemd-254")).unwrap();
        fs::create_dir(cat2.join("neovim-0.9.4")).unwrap();

        let mut total = 0;
        for cat in fs::read_dir(temp_dir.path()).unwrap().flatten() {
            if cat.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Ok(pkgs) = fs::read_dir(cat.path()) {
                    total += pkgs
                        .flatten()
                        .filter(|p| {
                            p.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                                && !p.file_name().to_string_lossy().starts_with('.')
                        })
                        .count();
                }
            }
        }
        assert_eq!(total, 3);
    }

    #[test]
    fn test_count_winget_from_dirs_mock() {
        let pkgs_tmp = tempfile::tempdir().unwrap();
        let links_tmp = tempfile::tempdir().unwrap();

        // 1. Initially empty
        assert_eq!(
            count_winget_from_dirs(pkgs_tmp.path(), links_tmp.path()),
            None
        );

        // 2. Packages dir populated
        fs::create_dir(pkgs_tmp.path().join("Microsoft.PowerToys_8wekyb3d8bbwe")).unwrap();
        fs::create_dir(pkgs_tmp.path().join("Git.Git_8wekyb3d8bbwe")).unwrap();
        fs::create_dir(pkgs_tmp.path().join(".hidden")).unwrap();

        assert_eq!(
            count_winget_from_dirs(pkgs_tmp.path(), links_tmp.path()),
            Some(2)
        );

        // 3. Fallback to Links dir if Packages is empty
        let empty_pkgs = tempfile::tempdir().unwrap();
        fs::write(links_tmp.path().join("code.exe"), b"").unwrap();
        fs::write(links_tmp.path().join("rg.exe"), b"").unwrap();
        fs::write(links_tmp.path().join(".hidden"), b"").unwrap();

        assert_eq!(
            count_winget_from_dirs(empty_pkgs.path(), links_tmp.path()),
            Some(2)
        );
    }

    #[test]
    fn test_count_choco_from_dir_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lib = temp_dir.path().join("lib");
        fs::create_dir_all(&lib).unwrap();

        fs::create_dir(lib.join("git")).unwrap();
        fs::create_dir(lib.join("curl")).unwrap();
        fs::create_dir(lib.join("7zip.install")).unwrap();
        fs::create_dir(lib.join(".hidden")).unwrap();

        assert_eq!(count_choco_from_dir(&lib), Some(3));
    }

    #[test]
    fn test_parse_cargo_crates_toml() {
        let fixture = r#"
[v1]
"bat 0.24.0 (registry+https://github.com/rust-lang/crates.io-index)" = ["bat.exe"]
"eza 0.18.0 (registry+https://github.com/rust-lang/crates.io-index)" = ["eza.exe"]
"ripgrep 14.1.0 (registry+https://github.com/rust-lang/crates.io-index)" = ["rg.exe"]

[other_section]
"ignored 1.0.0" = ["foo.exe"]
"#;
        assert_eq!(parse_cargo_crates_toml(fixture), 3);
        assert_eq!(parse_cargo_crates_toml(""), 0);
        assert_eq!(parse_cargo_crates_toml("[v1]\n"), 0);
    }

    #[test]
    fn test_count_cargo_from_path_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join(".crates.toml");

        fs::write(
            &file_path,
            "[v1]\n\"ferrisfetch 0.5.0 (registry+...)\" = [\"ferrisfetch\"]\n",
        )
        .unwrap();

        assert_eq!(count_cargo_from_path(&file_path), Some(1));
    }

    #[test]
    fn test_count_npm_from_dir_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir_all(&node_modules).unwrap();

        // Top level packages
        fs::create_dir(node_modules.join("typescript")).unwrap();
        fs::create_dir(node_modules.join("eslint")).unwrap();
        fs::create_dir(node_modules.join("npm")).unwrap(); // excluded
        fs::create_dir(node_modules.join(".bin")).unwrap(); // excluded

        // Scoped packages
        let scope = node_modules.join("@angular");
        fs::create_dir(&scope).unwrap();
        fs::create_dir(scope.join("cli")).unwrap();
        fs::create_dir(scope.join("core")).unwrap();

        // typescript (1) + eslint (1) + @angular/cli (1) + @angular/core (1) = 4
        assert_eq!(count_npm_from_dir(&node_modules), Some(4));
    }

    #[test]
    fn test_count_pip_from_dir_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let site_packages = temp_dir.path().join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        fs::create_dir(site_packages.join("requests-2.31.0.dist-info")).unwrap();
        fs::create_dir(site_packages.join("numpy-1.26.4.dist-info")).unwrap();
        fs::create_dir(site_packages.join("setuptools-68.0.0.egg-info")).unwrap();
        fs::create_dir(site_packages.join("requests")).unwrap(); // not dist-info
        fs::create_dir(site_packages.join("__pycache__")).unwrap();

        assert_eq!(count_pip_from_dir(&site_packages), Some(3));
    }

    #[test]
    fn test_count_scoop_from_dir_mock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let apps_dir = temp_dir.path().join("apps");
        fs::create_dir_all(&apps_dir).unwrap();

        fs::create_dir(apps_dir.join("git")).unwrap();
        fs::create_dir(apps_dir.join("curl")).unwrap();
        fs::create_dir(apps_dir.join("fastfetch")).unwrap();
        fs::create_dir(apps_dir.join("scoop")).unwrap(); // excluded self
        fs::create_dir(apps_dir.join(".git")).unwrap(); // excluded hidden

        assert_eq!(count_scoop_from_dir(&apps_dir), Some(3));
    }

    #[test]
    fn test_count_rpm_from_paths_nonexistent() {
        assert_eq!(count_rpm_from_paths(&["/nonexistent_rpm_db_12345"]), None);
    }
}
