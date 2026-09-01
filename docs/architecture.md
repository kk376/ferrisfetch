# KKFetch Architecture & Linux Detection Research

KKFetch is a fast, lightweight system information fetch tool written in Rust for Linux systems. It prioritizes sub-5 millisecond execution times, zero external runtime dependencies on standard distributions (Debian, Red Hat, Arch families), safe and panic-free fallback behavior, and clean terminal rendering.

---

## 1. System Overview & Objectives

### 1.1 Performance Targets
- **Execution Latency**: Complete fetch and render under 5 ms on standard hardware.
- **Resource Footprint**: Minimal heap allocation, single-pass virtual file parsing, zero spawned subprocesses during standard sysfs/procfs execution.
- **Binary Footprint**: Lean binary size with minimal external crate dependencies.

### 1.2 Core Architectural Principles
1. **Direct Kernel / Filesystem Probing**: Virtual filesystems (`/proc`, `/sys`) and direct POSIX C library calls (`libc`) provide all core system metrics. Spawning subprocesses (`uname`, `uptime`, `df`, `free`) is avoided because `fork`/`exec` introduces 1 to 4 ms of latency per process.
2. **Zero Shell Invocations**: String execution via `sh -c` is forbidden. Fallback commands (such as `lspci` or `rpm`) must be invoked via direct argument arrays in `std::process::Command`.
3. **Resilient Data Collection**: Every module collector returns `Option<T>` or `Result<T, CollectorError>`. Missing files, permission errors, virtualized environments, and minimal chroots degrade gracefully without panicking.
4. **Decoupled Architecture**: Probing logic, data modeling, formatting, and layout rendering are isolated into distinct crates or modules.

---

## 2. Module Breakdown & Technical Detection Mechanisms

### 2.1 Operating System & Distribution

#### Data Sources and Priority
1. `/etc/os-release` (systemd specification standard)
2. `/usr/lib/os-release` (immutable/stateless distribution fallback)
3. `/etc/debian_version`, `/etc/redhat-release`, `/etc/arch-release` (legacy fallbacks)
4. POSIX `libc::uname` fallback (`sysname` field)

#### File Format & Parser Requirements
The `/etc/os-release` file contains newline-separated `KEY=VALUE` assignments.
- Comment lines start with `#` and are skipped.
- Empty lines are skipped.
- Values can be unquoted, single-quoted (`'...'`), or double-quoted (`"..."`).
- Double-quoted strings can contain backslash-escaped characters (`\"`, `\\`, `\$`).

#### Extracted Keys
- `PRETTY_NAME`: Primary display string (e.g. `Ubuntu 24.04.4 LTS`, `Arch Linux`, `Fedora Linux 40 (Workstation Edition)`).
- `NAME`: Base distribution name if `PRETTY_NAME` is absent.
- `VERSION_ID` / `VERSION`: Appended to `NAME` if `PRETTY_NAME` is absent.
- `ID`: Lowercase canonical identifier used for ASCII logo matching (e.g. `ubuntu`, `fedora`, `arch`, `debian`, `rocky`, `almalinux`, `rhel`, `linuxmint`, `endeavouros`, `manjaro`).
- `ID_LIKE`: Space-separated list of upstream IDs used when a specific distro logo is not available.

#### Parser Implementation
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsInfo {
    pub display_name: String,
    pub distro_id: String,
    pub distro_like: Vec<String>,
}

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
            let mut val = v.trim();

            if (val.starts_with('"') && val.ends_with('"')) ||
               (val.starts_with('\'') && val.ends_with('\'')) {
                if val.len() >= 2 {
                    val = &val[1..val.len() - 1];
                }
            }

            match key {
                "PRETTY_NAME" => pretty_name = Some(val.to_string()),
                "NAME" => name = Some(val.to_string()),
                "VERSION" | "VERSION_ID" => {
                    if version.is_none() {
                        version = Some(val.to_string());
                    }
                }
                "ID" => id = Some(val.to_lowercase()),
                "ID_LIKE" => id_like = Some(val.to_lowercase()),
                _ => {}
            }
        }
    }

    let display_name = pretty_name
        .or_else(|| match (name, version) {
            (Some(n), Some(v)) => Some(format!("{} {}", n, v)),
            (Some(n), None) => Some(n),
            _ => None,
        })
        .unwrap_or_else(|| "Linux".to_string());

    let distro_id = id.unwrap_or_else(|| "linux".to_string());
    let distro_like = id_like
        .map(|s| s.split_whitespace().map(|x| x.to_string()).collect())
        .unwrap_or_default();

    OsInfo {
        display_name,
        distro_id,
        distro_like,
    }
}
```

---

### 2.2 Kernel, Hostname & User

#### Hostname & Kernel Architecture
`libc::uname` populates `utsname` in a single context switch without disk I/O:
- `release`: Kernel version string (e.g. `6.18.33.2-microsoft-standard-WSL2`, `6.10.3-arch1-2`).
- `machine`: CPU architecture (e.g. `x86_64`, `aarch64`, `riscv64`).
- `nodename`: System hostname.

```rust
use std::ffi::CStr;
use std::mem::MaybeUninit;

pub struct UnameInfo {
    pub kernel_release: String,
    pub architecture: String,
    pub hostname: String,
}

pub fn get_uname_info() -> Option<UnameInfo> {
    unsafe {
        let mut uts = MaybeUninit::<libc::utsname>::uninit();
        if libc::uname(uts.as_mut_ptr()) != 0 {
            return None;
        }
        let uts = uts.assume_init();

        let kernel_release = CStr::from_ptr(uts.release.as_ptr())
            .to_string_lossy()
            .into_owned();
        let architecture = CStr::from_ptr(uts.machine.as_ptr())
            .to_string_lossy()
            .into_owned();
        let hostname = CStr::from_ptr(uts.nodename.as_ptr())
            .to_string_lossy()
            .into_owned();

        Some(UnameInfo {
            kernel_release,
            architecture,
            hostname,
        })
    }
}
```

#### User Detection
1. Read `$USER` or `$LOGNAME` from environment.
2. Fallback: Query effective UID with `libc::geteuid()` and resolve username with `libc::getpwuid`.

#### Title Line Rendering
Format as `<user>@<hostname>` with an underline divider matching the text length:
```text
user@archlinux
--------------
```

---

### 2.3 System Uptime

#### Data Sources
- Primary: `/proc/uptime`
  First numerical token is total seconds as a float (e.g. `1978.59 23627.21`).
- Fallback: `libc::sysinfo(&mut info)` where `info.uptime` provides total seconds as an integer.

#### Formatting Rules
```rust
pub fn format_uptime(total_seconds: u64) -> String {
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if days > 0 {
        let day_label = if days == 1 { "day" } else { "days" };
        let hour_label = if hours == 1 { "hour" } else { "hours" };
        format!("{} {}, {} {}, {} mins", days, day_label, hours, hour_label, minutes)
    } else if hours > 0 {
        let hour_label = if hours == 1 { "hour" } else { "hours" };
        format!("{} {}, {} mins", hours, hour_label, minutes)
    } else {
        format!("{} mins", minutes)
    }
}
```

---

### 2.4 CPU Information

#### Data Source: `/proc/cpuinfo`
- **x86 / x86_64**: Read `model name` lines.
- **ARM / AArch64**: Read `Model`, `Hardware`, or `Processor` lines.
- **Core Count**: Count occurrences of `processor\t:` or query `libc::sysconf(libc::_SC_NPROCESSORS_ONLN)`.
- **Physical Sockets**: Count unique values of `physical id` across processors (defaulting to 1).

#### String Sanitization
CPU model strings from manufacturers contain redundant branding tokens. KKFetch strips:
- `(R)`, `(TM)`, `(tm)`
- `CPU @ ...` clock speed suffixes
- `Processor`, `Dual-Core`, `Quad-Core`, `Six-Core`, `Eight-Core`
- Multiple consecutive spaces

```rust
pub fn clean_cpu_model(raw: &str) -> String {
    let cleaned = raw
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("(tm)", "")
        .replace("CPU", "")
        .replace("Processor", "");

    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    parts.join(" ")
}
```

Example output: `AMD Ryzen 7 7700X with Radeon Graphics (16)`

---

### 2.5 GPU Detection

#### Detection Hierarchy
1. **Sysfs PCI Class Scan** (`/sys/bus/pci/devices/*`):
   - Check the `class` file for base class `0x03`:
     - `0x030000`: VGA compatible controller
     - `0x030200`: 3D controller
     - `0x038000`: Display controller
   - Read `vendor` and `device` IDs (e.g. `0x10de` $\rightarrow$ NVIDIA, `0x1002` $\rightarrow$ AMD, `0x8086` $\rightarrow$ Intel, `0x1af4` $\rightarrow$ VirtIO, `0x1414` $\rightarrow$ Microsoft).
   - Read `label` or `product` if present in the device folder.
2. **Direct `lspci` Invocation Fallback**:
   - If sysfs does not provide a readable model string, call `lspci -mm -d ::0300` directly via `Command::new("lspci")`.
   - Parse columnated output: `Slot "Class" "Vendor" "Device"`.
3. **Headless / Container Handling**:
   - When no display controllers are found, the GPU entry is omitted cleanly.

---

### 2.6 Memory (RAM)

#### Data Source: `/proc/meminfo`
Read the first 1024 bytes of `/proc/meminfo`.

#### Calculation Logic
- `MemTotal`: Total usable RAM.
- `MemAvailable`: Kernel estimate of memory available for starting new applications without swapping.
- If `MemAvailable` is present:
  $$\text{Used} = \text{MemTotal} - \text{MemAvailable}$$
- Fallback for kernels older than 3.14:
  $$\text{Used} = \text{MemTotal} - \text{MemFree} - \text{Buffers} - \text{Cached} - \text{SReclaimable} + \text{Shmem}$$

#### Unit Formatting
- Express values in GiB ($1\text{ GiB} = 1024 \times 1024\text{ kB}$):
  `2.14 GiB / 7.36 GiB (29%)`
- If total memory is under 1 GiB, format in MiB:
  `412 MiB / 980 MiB (42%)`

---

### 2.7 Disk Usage

#### Data Source: `libc::statvfs`
Target directory is configurable via `--disk-path`, defaulting to root `/`.

```rust
use std::ffi::CString;
use std::mem::MaybeUninit;

pub struct DiskUsage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage: u8,
}

pub fn get_disk_usage(path: &str) -> Option<DiskUsage> {
    let c_path = CString::new(path).ok()?;
    unsafe {
        let mut stat = MaybeUninit::<libc::statvfs>::uninit();
        if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) != 0 {
            return None;
        }
        let stat = stat.assume_init();

        let block_size = if stat.f_frsize > 0 { stat.f_frsize } else { stat.f_bsize } as u64;
        let total_bytes = stat.f_blocks as u64 * block_size;
        let free_bytes = stat.f_bavail as u64 * block_size;
        let used_bytes = total_bytes.saturating_sub(stat.f_bfree as u64 * block_size);

        if total_bytes == 0 {
            return None;
        }

        let percentage = ((used_bytes as f64 / total_bytes as f64) * 100.0).round() as u8;

        Some(DiskUsage {
            total_bytes,
            used_bytes,
            free_bytes,
            percentage,
        })
    }
}
```

Format: `32.4 GiB / 250.0 GiB (13%)`

---

### 2.8 Shell Detection

#### Detection Strategy
1. **Process Inspection via `/proc`**:
   - Find parent process ID ($PPID$) from `/proc/self/status` or `/proc/self/stat`.
   - Read `/proc/<PPID>/comm` or target of `/proc/<PPID>/exe`.
   - If the parent process is `cargo` or `kkfetch`, ascend to the grandparent process.
2. **Environment Fallback**:
   - Read `$SHELL` and extract the file stem (e.g. `/bin/bash` $\rightarrow$ `bash`).

---

### 2.9 Terminal Detection

#### Detection Hierarchy
1. `$TERM_PROGRAM` with `$TERM_PROGRAM_VERSION` (e.g. `vscode 1.96.0`, `ghostty`, `WezTerm`, `iTerm.app`).
2. Terminal environment variables:
   - `$ALACRITTY_LOG` / `$ALACRITTY_WINDOW_ID` $\rightarrow$ `Alacritty`
   - `$KITTY_PID` / `$KITTY_WINDOW_ID` $\rightarrow$ `kitty`
   - `$KONSOLE_VERSION` $\rightarrow$ `Konsole`
   - `$WT_SESSION` $\rightarrow$ `Windows Terminal`
   - `$FOOT_PID` $\rightarrow$ `foot`
3. Process ancestry scan:
   - Traverse parent processes from `/proc/<pid>/stat` to check for known terminal process names (`gnome-terminal-server`, `konsole`, `alacritty`, `kitty`, `wezterm-gui`, `foot`, `xterm`).
4. Fallback: `$TERM` (e.g. `xterm-256color`).

---

### 2.10 Desktop Environment & Window Manager

#### Desktop Environment (DE)
- Read `$XDG_CURRENT_DESKTOP` (e.g. `GNOME`, `KDE`, `XFCE`, `Cinnamon`, `MATE`).
- Fallback: `$DESKTOP_SESSION`.

#### Window Manager (WM) & Session Type
- Session type: `$XDG_SESSION_TYPE` (`wayland`, `x11`, `tty`).
- Wayland signatures:
  - `$SWAYSOCK` $\rightarrow$ `Sway`
  - `$HYPRLAND_INSTANCE_SIGNATURE` $\rightarrow$ `Hyprland`
  - `$WAYFIRE_CONFIG_FILE` $\rightarrow$ `Wayfire`
  - `$RIVER_SOCKET` $\rightarrow$ `River`
- Process scan fallback:
  - Check running processes for known WMs (`i3`, `bspwm`, `awesome`, `dwm`, `openbox`, `xmonad`, `qtile`, `mutter`, `kwin`).
- Headless check:
  - If no display variable exists and session type is `tty` or unset, display `Headless` or omit if graphical modules are disabled.

---

### 2.11 Package Count (Offline & Fast)

All package queries operate on local metadata files without network access or package database locking.

| Distribution Family | Detection Strategy | Complexity |
| :--- | :--- | :--- |
| **Debian / Ubuntu / Mint** | Count lines matching `Status: install ok installed` in `/var/lib/dpkg/status`. | Single buffered scan ($\approx 1\text{ ms}$) |
| **Arch / Endeavour / Manjaro** | Count directory entries in `/var/lib/pacman/local/` (excluding `.` and `..`). | Single `read_dir` count ($\approx 0.3\text{ ms}$) |
| **Fedora / RHEL / Rocky / Alma** | Read entry count from `/var/lib/rpm/Packages` or count lines in `rpm -qa`. | Quick local scan |
| **Flatpak** | Count subdirectories in `/var/lib/flatpak/app/` and `~/.local/share/flatpak/app/`. | Two directory checks ($\approx 0.2\text{ ms}$) |
| **Snap** | Count squashfs images in `/var/lib/snapd/snaps/` or directories in `/snap/`. | Directory check ($\approx 0.2\text{ ms}$) |

Combined output format: `1018 (dpkg), 6 (flatpak)`

---

## 3. System Architecture & Module Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│                          CLI Entry                           │
│      (Clap v4: Parse arguments, flags, custom options)       │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                        FetchContext                          │
│  - Terminal width (libc::ioctl TIOCGWINSZ)                   │
│  - Color enablement (std::io::IsTerminal + --no-color)       │
│  - Cached OS metadata (/etc/os-release)                      │
│  - Module enablement / disablement filters                   │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                       Module Registry                        │
│  Ordered collection:                                         │
│  [ Title, OS, Host, Kernel, Uptime, Packages, Shell,         │
│    Desktop, Terminal, CPU, GPU, Memory, Disk, Colors ]       │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                      Layout & Renderer                       │
│  - Match ASCII logo from distro_id or --logo flag            │
│  - Strip ANSI codes for precise column alignment calculation │
│  - Two-column side-by-side layout (or stacked if width < 60) │
│  - Terminal 8-color palette bar footer                       │
└──────────────────────────────────────────────────────────────┘
```

---

### 3.1 Data Structures & Trait Abstractions

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleId {
    Title,
    Os,
    Host,
    Kernel,
    Uptime,
    Packages,
    Shell,
    Desktop,
    Terminal,
    Cpu,
    Gpu,
    Memory,
    Disk,
    Colors,
}

impl ModuleId {
    pub fn all() -> &'static [ModuleId] {
        &[
            ModuleId::Title,
            ModuleId::Os,
            ModuleId::Host,
            ModuleId::Kernel,
            ModuleId::Uptime,
            ModuleId::Packages,
            ModuleId::Shell,
            ModuleId::Desktop,
            ModuleId::Terminal,
            ModuleId::Cpu,
            ModuleId::Gpu,
            ModuleId::Memory,
            ModuleId::Disk,
            ModuleId::Colors,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleId::Title => "title",
            ModuleId::Os => "os",
            ModuleId::Host => "host",
            ModuleId::Kernel => "kernel",
            ModuleId::Uptime => "uptime",
            ModuleId::Packages => "packages",
            ModuleId::Shell => "shell",
            ModuleId::Desktop => "desktop",
            ModuleId::Terminal => "terminal",
            ModuleId::Cpu => "cpu",
            ModuleId::Gpu => "gpu",
            ModuleId::Memory => "memory",
            ModuleId::Disk => "disk",
            ModuleId::Colors => "colors",
        }
    }

    pub fn from_str(s: &str) -> Option<ModuleId> {
        match s.trim().to_lowercase().as_str() {
            "title" => Some(ModuleId::Title),
            "os" => Some(ModuleId::Os),
            "host" => Some(ModuleId::Host),
            "kernel" => Some(ModuleId::Kernel),
            "uptime" => Some(ModuleId::Uptime),
            "packages" | "pkgs" => Some(ModuleId::Packages),
            "shell" => Some(ModuleId::Shell),
            "desktop" | "de" | "wm" => Some(ModuleId::Desktop),
            "terminal" | "term" => Some(ModuleId::Terminal),
            "cpu" => Some(ModuleId::Cpu),
            "gpu" => Some(ModuleId::Gpu),
            "memory" | "mem" => Some(ModuleId::Memory),
            "disk" => Some(ModuleId::Disk),
            "colors" | "palette" => Some(ModuleId::Colors),
            _ => None,
        }
    }
}

pub struct ModuleOutput {
    pub id: ModuleId,
    pub label: String,
    pub value: String,
    pub custom_rendered: Option<String>,
}

pub struct FetchContext {
    pub term_width: u16,
    pub enable_color: bool,
    pub os_info: OsInfo,
    pub disk_target_path: String,
    pub active_modules: Vec<ModuleId>,
}

pub trait Collector: Send + Sync {
    fn id(&self) -> ModuleId;
    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput>;
}
```

---

### 3.2 Terminal Layout & ANSI Handling

#### Column Alignment Algorithm
1. Retrieve terminal width via `libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws)`. Default to 80 if unavailable.
2. If width $< 60$: Stack logo above information or suppress logo.
3. If width $\ge 60$:
   - Split ASCII logo into lines. Compute `max_logo_width` using visible character count (excluding ANSI escapes).
   - Compute total lines: `max(logo_lines.len(), info_lines.len())`.
   - For each line $i$, format:
     $$\text{line} = \text{pad\_visible}(\text{logo}[i], \text{max\_logo\_width}) + \text{"   "} + \text{info}[i]$$

#### ANSI Visible Length Calculator
```rust
pub fn visible_width(s: &str) -> usize {
    let mut in_escape = false;
    let mut len = 0;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}
```

---

### 3.3 ASCII Logo Specifications & Color Palettes

KKFetch contains dedicated, compact ASCII art representations for major Linux distributions and the Ferris mascot.

#### 1. Ferris (Rust Crab Mascot) - Default / Generic
Primary Color: Coral Red (`\x1b[38;5;208m`), Accent: White (`\x1b[37m`)
```text
    _~^~^~_
\) /  o o  \ (/
  '_   -   _'
  / '-----' \
```

#### 2. Debian
Primary Color: Red (`\x1b[38;5;196m`), Accent: White (`\x1b[37m`)
```text
  _____
 / ____|
| |  __
| | |_ |
| |__| |
 \_____|
```

#### 3. Ubuntu
Primary Color: Orange (`\x1b[38;5;208m`), Accent: White (`\x1b[37m`)
```text
         _
     ---(_)
 _/  ---  \
(_) |   |
  \  --- _/
     ---(_)
```

#### 4. Linux Mint
Primary Color: Mint Green (`\x1b[38;5;46m`), Accent: White (`\x1b[37m`)
```text
 ___________
|_          \
  | | _____ |
  | | | | | |
  | | | | | |
  | \_____/ |
  \_________/
```

#### 5. Fedora
Primary Color: Blue (`\x1b[38;5;33m`), Accent: Cyan (`\x1b[38;5;81m`)
```text
      _____
     /   __)-/
    |  /  ___
 __ | |  / __|
(  \| | ( (_) |
 \____/  \___/
```

#### 6. Arch Linux
Primary Color: Cyan (`\x1b[38;5;67m`), Accent: Light Cyan (`\x1b[38;5;123m`)
```text
      /\
     /  \
    /\   \
   /      \
  /   ,,   \
 /__ /  \ __\
```

#### 7. Red Hat Enterprise Linux (RHEL)
Primary Color: Bright Red (`\x1b[38;5;196m`), Accent: Dark Gray (`\x1b[90m`)
```text
      .---.
     / /"\ \
    | |   | |
 .-.-\ \_/ /.-.-.
/  ___"""""___  \
\__(_________)__/
```

#### 8. Rocky Linux
Primary Color: Emerald Green (`\x1b[38;5;35m`), Accent: Light Green (`\x1b[38;5;84m`)
```text
    .---.
   /     \
  |  /\   |
  |  \/   |
   \     /
    '---'
```

#### 9. AlmaLinux
Primary Color: Cyan (`\x1b[38;5;39m`), Accent: Yellow (`\x1b[38;5;220m`)
```text
   __o__
  /     \
 |   O   |
  \  |  /
   '-o-'
```

#### 10. EndeavourOS
Primary Color: Purple (`\x1b[38;5;127m`), Accent: Red (`\x1b[38;5;197m`)
```text
      / \
     /   \
    /  /\ \
   /  /  \ \
  /__/    \_\
```

#### 11. Manjaro
Primary Color: Green (`\x1b[38;5;34m`), Accent: Bright Green (`\x1b[38;5;47m`)
```text
||||||||| ||||
||||||||| ||||
||||      ||||
|||| |||| ||||
|||| |||| ||||
|||| |||| ||||
```

#### 12. Generic Linux (Tux)
Primary Color: Yellow (`\x1b[38;5;220m`), Accent: White (`\x1b[37m`)
```text
   .--.
  |o_o |
  |:_/ |
 //   \ \
(|     | )
/'\_   _/`\
\___)=(___/
```

---

## 4. Dependencies & Crate Strategy

```toml
[package]
name = "kkfetch"
version = "0.8.6"
edition = "2021"
rust-version = "1.75.0"

[dependencies]
# CLI parsing
clap = { version = "4.5", features = ["derive"] }

# Direct POSIX calls
libc = "0.2"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"
```

### Evaluation: Custom Parsers vs `regex` Crate
- `regex` pulls in multiple crates (`regex-syntax`, `regex-automata`, `aho-corasick`) and adds compilation time and binary weight.
- Every text structure parsed in KKFetch (`os-release`, `cpuinfo`, `meminfo`, `uptime`, `dpkg/status`) consists of simple delimited key-value pairs or token lists.
- Using `split_once(':')`, `split_once('=')`, and standard slice iterators avoids all dependencies, compiles instantly, and processes files in microseconds.

### Minimum Supported Rust Version (MSRV)
- **Target**: Rust 1.75.0+
- Allows standard library `std::io::IsTerminal` without external helper crates.

---

## 5. Security & Error Handling

1. **Path Safety**: All virtual file paths are hardcoded constants pointing to trusted `/proc`, `/sys`, `/etc`, and `/var` locations.
2. **Subprocess Isolation**: Where subprocesses are executed (`lspci`, `rpm`), arguments are supplied as a static array to `std::process::Command::new(...)`. Shell expansion and command injection are impossible.
3. **No Unwraps in Collector Paths**: Every collector returns `None` or an empty vector on I/O failures, permission denial, or unexpected formats.
4. **Immutable / Read-Only Safety**: File operations use read-only descriptors (`std::fs::File::open`).

---

## 6. Testing Strategy

### 6.1 Parser Unit Tests with Static Fixtures
Unit tests parse static string fixtures representing outputs from diverse environments:
- `tests/fixtures/os_release/ubuntu_24_04.txt`
- `tests/fixtures/os_release/arch_rolling.txt`
- `tests/fixtures/os_release/fedora_40.txt`
- `tests/fixtures/os_release/rocky_9.txt`
- `tests/fixtures/cpuinfo/intel_i7_10750h.txt`
- `tests/fixtures/cpuinfo/amd_ryzen_7700x.txt`
- `tests/fixtures/cpuinfo/arm64_raspberry_pi.txt`
- `tests/fixtures/meminfo/standard_16gb.txt`
- `tests/fixtures/uptime/standard.txt`
- `tests/fixtures/dpkg/status_sample.txt`

### 6.2 Layout Alignment & ANSI Tests
- Test column layout with mismatched logo and info line counts.
- Validate ANSI visible width calculations across varying terminal widths.

### 6.3 CLI Integration Tests
Using `assert_cmd`:
```rust
#[test]
fn test_no_color_flag() {
    let mut cmd = Command::cargo_bin("kkfetch").unwrap();
    cmd.arg("--no-color");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains("\x1b["));
}

#[test]
fn test_list_modules_flag() {
    let mut cmd = Command::cargo_bin("kkfetch").unwrap();
    cmd.arg("--list-modules");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("os"))
        .stdout(predicates::str::contains("cpu"))
        .stdout(predicates::str::contains("memory"));
}
```
