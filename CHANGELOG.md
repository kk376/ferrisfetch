# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.3] - 2026-08-29

### Fixed
- **Mobile APU & SoC Integrated GPU Classification**: Expanded iGPU pattern detection to properly recognize AMD Radeon 610M (Mendocino APUs), 600/700/800-series mobile APU processors, Intel Arc iGPUs, and mobile SoCs as `[Integrated]` graphics on single-GPU and hybrid configurations (reported by [@Laynsb](https://github.com/Laynsb)).

## [0.11.2] - 2026-08-29

### Optimized
- **Instantaneous Battery Probing & Tmpfs Runtime Cache**: Implemented single-pass sysfs scan with non-blocking 15-second TTL tmpfs runtime caching in `$XDG_RUNTIME_DIR/ferrisfetch_battery.cache` to completely eliminate hardware ACPI EC bus latency spikes on laptops, guaranteeing deterministic sub-3ms parallel execution.

## [0.11.1] - 2026-08-29

### Added
- **Dedicated GPU VRAM Memory Probing**: Zero-fork GPU memory scanning parsing `mem_info_vram_total` from sysfs and 64-bit prefetchable memory apertures from `/sys/bus/pci/devices/*/resource` to report dedicated VRAM size (e.g. `512 MiB`, `4 GiB`).
- **Windows Display Adapter VRAM Parsing**: Extracted `HardwareInformation.qwMemorySize` and `HardwareInformation.MemorySize` directly from Windows display adapter registry classes.
- **Updated Statistical Benchmarks**: Re-benchmarked on Fedora Linux 44 with sub-3ms average latency across all 27 active modules.

## [0.11.0] - 2026-08-29

### Added
- **Zero-Dependency TOML Configuration System**: Added hierarchical configuration parser loading `~/.config/ferrisfetch/config.toml` (and `/etc/ferrisfetch/config.toml`) supporting module ordering, custom labels, logo presets, colors, separator formatting, and per-module settings.
- **Custom Info Module Loader & Script Plugin System**: Parallel execution of external scripts in `~/.config/ferrisfetch/plugins/` and custom command modules declared in `config.toml`.
- **CPU Topology & Dual Live Clock Frequency**: Physical cores vs logical threads reporting (`6c 12t`) alongside live instantaneous core clock and rated maximum turbo boost frequency (`@ 4.351GHz [4.60GHz max]`).
- **Extended Zero-Fork Sysfs EDID DRM Parser**: Decodes 128-byte EDID binary payloads from `/sys/class/drm/card*-*/edid` to extract PNP monitor manufacturer code, diagonal physical inch size, native resolution, refresh rate, and connector type.
- **System Font, WM Theme, and Terminal Font Collectors**: Native parsers for GTK 2/3/4 `settings.ini`, GNOME GSettings/dconf, KDE `kdeglobals`, and Kitty, Alacritty, and Foot dotfiles.
- **Cursor Theme & Pixel Size Collector**: Resolves cursor theme and size from GTK, GNOME, and KDE configurations.
- **Terminal Emulator Version Detection**: Probes version numbers across all major terminal emulators (Kitty, Alacritty, Foot, WezTerm, Ghostty, GNOME Terminal, GNOME Console, Konsole, XFCE Terminal, MATE Terminal, Tilix, Terminator, tmux, Zellij, Rio, Contour, BlackBox, Ptyxis, xterm, VS Code).
- **Deep Terminal Capability Probing**: Real-time detection of 24-bit TrueColor, UTF-8 Unicode, and Nerd Fonts support.
- **Multi-OS Support & Dedicated ASCII Art Logos**: Added brand-colored ASCII logos and native OS detection for **Android** (Termux), **macOS** (Darwin), **OpenBSD**, and **NetBSD**.
- **Microsecond Profiling Mode (`--timings`)**: Per-module latency diagnostics showing individual execution durations and wall-clock times.

## [0.10.2] - 2026-08-24

### Fixed
- **Permanent Dual-Boot RTC Skew Detection**: Inspects `/etc/adjtime` for `LOCAL` mode to deterministically normalize rootfs birth timestamps on dual-boot Windows/Linux machines regardless of current uptime or time-of-day.
- **Calendar-Day Relative Time Resolution**: Computes installation age based on localized calendar day midnight boundaries, correctly displaying `yesterday` (instead of `today`) when crossed over midnight.
- **Deduplicated Desktop & Window Manager Display**: Suppresses redundant default compositor annotations in Desktop module (e.g. `GNOME 50.4 (Wayland)` instead of repeating `Mutter` alongside the dedicated `WM` module).

## [0.10.1] - 2026-08-24

### Fixed
- **Clean GPU Marketing Model Resolution**: Automatically extracts consumer product brand names from bracketed PCI hardware identifiers (e.g. `[GeForce RTX 2050]`, `[Radeon 680M]`), eliminating internal raw silicon codenames (`GA107`, `Rembrandt`).
- **Dual-Boot RTC Installer Skew Normalization**: Automatically detects and normalizes filesystem birth time offsets caused by Live USB installer hardware clock assumptions on dual-boot Windows/Linux systems.

## [0.10.0] - 2026-08-23

### Optimized
- **Zero-Subprocess Windows Shell & Terminal Resolution**: Replaced process spawning with native Win32 Toolhelp32 process snapshot ancestry traversal (`CreateToolhelp32Snapshot`), eliminating 200ms–400ms startup latency.
- **Accurate Windows Shell Detection**: Resolved Command Prompt (`cmd.exe`) version directly from Registry `CurrentBuildNumber` + `UBR` (`CMD 10.0.<build>.<ubr>`).
- **Dedicated Windows Package Manager Pipeline**: Added Scoop package manager discovery (`%USERPROFILE%\scoop\apps`) and bypassed Linux filesystem probes on Windows.

## [0.9.9] - 2026-08-23

### Fixed
- **Vendored Package Checksum Preservation**: Preserved crate package checksum hashes while zeroing file-level maps in `.cargo-checksum.json` for full compatibility with Cargo lockfile verification during offline builds.

## [0.9.8] - 2026-08-23

### Fixed
- **Launchpad PPA Checksum Normalization**: Fixed vendored crate `.cargo-checksum.json` generation and cargo vendor path configuration for Ubuntu Noble offline builds.

## [0.9.7] - 2026-08-23

### Optimized
- **RPM MTIME Package Caching**: Added persistent mtime-based disk caching for RPM database queries, slashing Fedora package query times from ~1.5s to <0.05ms.
- **DRM-First Display Resolution**: Prioritized direct kernel sysfs DRM connector parsing over graphical server roundtrips, eliminating unnecessary `xrandr` / `wlr-randr` subprocesses on Wayland.
- **Desktop Environment Caching**: Added persistent caching for desktop environment versions to eliminate heavy runtime subprocess spawning (e.g. `gnome-shell --version`).

## [0.9.6] - 2026-08-23

### Added
- **WSL Host Version Discovery**: Probes and reports host WSL version on the Host line (e.g. `Host: Windows Subsystem for Linux - 2.7.12.0`).
- **Vendored Checksum Normalization**: Optimized Debian/PPA offline cargo build compatibility.

## [0.9.5] - 2026-08-22

### Added
- **WSLg Version Discovery**: Probes and reports the active WSLg version from `/mnt/wslg/versions.txt` (e.g. `WM: WSLg 1.0.73.2 (Wayland)`).
- **GPU Type Classification**: Automatically identifies and annotates `[Integrated]` vs `[Discrete]` GPUs across hybrid laptop and multi-GPU workstation setups.

## [0.9.0] - 2026-08-22

### Added
- **Enhanced ASCII Distro Art**: High-contrast white outer framing with brand-colored inner emblems across all 26+ distributions.
- **WSL2 Storage Resolution**: Normalized 9p/drvfs virtualization filesystem mappings to native NTFS for Windows drive mounts.

## [0.8.5] - 2026-08-22

### Added
- **Neofetch Dual-Tone ASCII Art Suite**: Integrated the complete, classic multi-color ASCII art logo suite from Neofetch across all 26 supported distributions and operating systems (Ubuntu, Fedora, Arch Linux, Debian, Linux Mint, NixOS, openSUSE, Gentoo, Void, Pop!_OS, RHEL, Rocky, AlmaLinux, EndeavourOS, Manjaro, Alpine, Kali, FreeBSD, Slackware, Artix, Zorin, Windows 11/10/7, Tux, and Ferris the Crab).
- **Dual-Tone Color Token Rendering Engine**: Added internal ANSI color token parsing (`{p}` for primary distro color, `{a}` for accent/white highlights, `{0}` for reset) with automated ANSI stripping for `--no-color` mode and zero-distortion column alignment.
- **Filesystem Type Discovery on Disks**: Enumerates filesystem types (e.g. `ext4`, `btrfs`, `ntfs`, `9p`, `vfat`, `zfs`) across all mounted storage partitions on Linux (`/proc/mounts`) and Windows (`GetVolumeInformationW`) with zero subprocess overhead (suggested by [@Laynsb](https://github.com/Laynsb)).
- **ZRAM Compression Algorithm Discovery on Swap**: Detects active in-memory swap compression algorithms from `/sys/block/zram*/comp_algorithm` (e.g. `Swap: 0.00 GiB / 4.00 GiB (0%) - LZ4`) on ZRAM-enabled distributions (Fedora, Pop!_OS, ChromeOS, Android), while leaving traditional swap files/partitions clean (suggested by [@Laynsb](https://github.com/Laynsb)).

## [0.8.0] - 2026-08-22

### Added
- **Filesystem Type Discovery on Disks**: Enumerates filesystem types (e.g. `ext4`, `btrfs`, `ntfs`, `9p`, `vfat`, `zfs`) across all mounted storage partitions on Linux (`/proc/mounts`) and Windows (`GetVolumeInformationW`) with zero subprocess overhead (suggested by [@Laynsb](https://github.com/Laynsb)).
- **ZRAM Compression Algorithm Discovery on Swap**: Detects active in-memory swap compression algorithms from `/sys/block/zram*/comp_algorithm` (e.g. `Swap: 0.00 GiB / 4.00 GiB (0%) - LZ4`) on ZRAM-enabled distributions (Fedora, Pop!_OS, ChromeOS, Android), while leaving traditional swap files/partitions clean (suggested by [@Laynsb](https://github.com/Laynsb)).
- **High-Fidelity Distro ASCII Art Logos**: Redesigned all distribution and OS ASCII art logos with proportionally taller, high-contrast silhouettes across Ubuntu, Fedora, Debian, Arch Linux, Linux Mint, RHEL, Rocky, AlmaLinux, openSUSE, Gentoo, Void, Pop!_OS, NixOS, Kali, FreeBSD, Windows 11/10/7, and Tux (suggested by [@Laynsb](https://github.com/Laynsb)).

## [0.7.0] - 2026-08-22

### Added
- **Localized Installation Timestamps**: System installation date and time (`Installed:`) is now automatically formatted in the user's local timezone (including daylight saving time adjustments) using native OS APIs (`localtime_r` / `tm_gmtoff` on POSIX systems and `GetTimeZoneInformation` on Windows), replacing raw UTC+0 display (suggested by [@Laynsb](https://github.com/Laynsb)).

## [0.6.0] - 2026-08-22

### Added
- **Native Windows NT Platform Support**: Full native Win32 execution without WSL or external runtime dependencies.
- **Win32 Hardware & System Probers**:
  - **OS**: Probes Windows product name, display version, and build number from registry (`HKLM\...\CurrentVersion`), with automatic Windows 11 upgrade build detection.
  - **Kernel**: Reports `Windows NT <version>.<build>`.
  - **Host**: Resolves manufacturer, product name, and BIOS version from `HKLM\HARDWARE\DESCRIPTION\System\BIOS`.
  - **CPU**: Probes processor name and clock frequency from central processor registry keys and queries active logical processor count.
  - **Memory & Swap**: Queries physical RAM and pagefile capacity via Win32 `GlobalMemoryStatusEx`.
  - **GPU**: Discovers display adapters and dedicated VRAM from video controller registry keys.
  - **Disks**: Enumerates Windows drive letters (`C:\`, `D:\`, etc.) and storage metrics via Win32 `GetDiskFreeSpaceExW`.
  - **Battery**: Queries battery capacity, charging status, and AC line state via `GetSystemPowerStatus`.
  - **Uptime**: Computes system elapsed uptime via `GetTickCount64`.
  - **Install Date**: Formats system installation timestamp from registry records with relative time delta.
  - **Theme**: Detects Windows Light/Dark mode preference from personalization registry keys.
  - **Desktop & WM**: Reports `Windows Explorer` and `Desktop Window Manager (DWM)`.
- **Windows Package Managers**: Native package counting for **WinGet** (`%LOCALAPPDATA%\Microsoft\WinGet`), **Chocolatey** (`C:\ProgramData\chocolatey\lib`), and **Cargo** (`.crates.toml`).
- **Windows Shells & Terminals**: Detection and version parsing for **PowerShell 7** (`pwsh`), **Windows PowerShell 5.1** (`powershell`), **Command Prompt** (`cmd`), **Nushell** (`nu`), **Windows Terminal** (`$WT_SESSION`), and **Console Window Host** (`ConHost`).
- **Windows ASCII Art Logos**: Added high-resolution ANSI logos for **Windows 11**, **Windows 10**, and **Classic Windows / Windows 7**.
- **Distribution Channels**: Added packaging manifests for **WinGet**.

## [0.5.0] - 2026-08-19

### Added
- **System Installation Date Module (`Installed`)**: Probes root filesystem creation timestamp (`stx_btime`) and distribution installer records, formatting as intuitive `DD Mon YYYY, hh:mm AM/PM (X days ago)` (e.g. `Installed: 16 Aug 2026, 02:32 PM (3 days ago)`). Suggested by [@Laynsb](https://github.com/Laynsb).
- **Universal Terminal Detection Expansion**: Added native detection signatures and version resolution for **Ptyxis** (`$PTYXIS_VERSION`), **Ghostty** (`$GHOSTTY_VERSION`), **GNOME Console** (`kgx`), **BlackBox**, **Contour**, **Rio**, **Yakuake**, **Guake**, **LXTerminal**, **MATE Terminal**, **QTerminal**, **Deepin Terminal**, **Pantheon Terminal**, **Warp**, and **Zellij**.
- **Desktop Environment Version Resolution**: Appends detected DE versions from metadata files and version queries (e.g. `GNOME 50.1`, `KDE Plasma 6.1`, `XFCE 4.18`, `MATE 1.28`, `Cinnamon 6.0`).
- **Intel iGPU & Linux GPU Clock Speed**: Probes maximum graphics clock frequency from `/sys/class/drm/card*/gt_max_freq_mhz` and hwmon sysfs (e.g. `GPU0: Intel HD Graphics 620 @ 1.000GHz`).
- **Wayland Display Refresh Rate**: Added native refresh rate resolution for Wayland compositors via `wlr-randr` and DRM sysfs.

## [0.4.2] - 2026-08-19

### Fixed
- **Android / Termux Disk Filtering**: Filtered out Android internal loop mounts and read-only system subsystems (`/apex`, `/bootstrap-apex`, `/data/app`, `/data/user`, `/metadata`, `/product`, `/vendor`, `/system`). Only actual user storage partitions (`/`, `/data`, `/storage/*`) are listed in Termux.

## [0.4.1] - 2026-08-19

### Changed
- **Clean Battery Formatting**: Streamlined battery output across WSL and native Linux to display standard metrics and AC connection state (e.g. `Battery: 97% [AC Connected]`) without redundant hypervisor model strings.

## [0.4.0] - 2026-08-19

### Added
- **OS Architecture**: Displays system architecture alongside distribution name (e.g. `OS: Ubuntu 24.04.4 LTS x86_64`).
- **Shell Version**: Resolves active shell version (e.g. `Shell: zsh 5.9`, `Shell: bash 5.2.21`).
- **Display Resolution & Refresh Rate**: Probes connected displays and refresh rates (e.g. `Display: 1920x1080 @ 60Hz`).
- **Window Manager Module (`WM`)**: Resolves active window managers including Mutter, KWin, Xfwm4, Sway, Hyprland, and `WSLg (Weston)`.
- **GPU VRAM & Clock Frequency**: Displays GPU memory capacity and max graphics clock (e.g. `GPU0: Intel Iris Xe Graphics (1 GiB) @ 1.400GHz`, `GPU1: NVIDIA GeForce RTX 4090 (24 GiB) @ 2.520GHz`).
- **Swap Memory Module**: Displays total and active used swap partition/file memory.
- **Partition Disk Enumeration**: Discovers all active physical and virtual partitions labeled sequentially (`Disk0`, `Disk1`, `Disk2`), formatting WSL Windows drives directly (e.g. `(C)`, `(D)`).
- **Physical Battery Detection**: Probes battery percentage and status from sysfs while automatically filtering out Microsoft Hyper-V virtual batteries in WSL.
- **Local IP Module**: Probes primary local IPv4 address via standard POSIX interface enumeration without subprocesses.

## [0.3.0] - 2026-08-19

### Added
- **Multi-Socket CPU Scaling**: Formats multi-socket CPU systems as `<n>x <CPU Name> (<Total Threads>)` (e.g. `3x AMD EPYC 9654 (384)`).
- **CPU Clock Speed**: Added frequency resolution (`@ nGHz`) from `/proc/cpuinfo` and `cpufreq` sysfs.
- **Dynamic Sequential GPU Indexing**: Assigns sequential indices (`GPU0`, `GPU1`, `GPU2`, ...) without skipping numbers.
- **iGPU `GPU0` Priority**: Integrated graphics always occupy `GPU0` and scale across multi-socket systems (`GPU0: <n>x <iGPU Name>`).
- **dGPU Automatic Grouping**: Automatically groups identical discrete GPUs into a single line (`GPU<index>: <n>x <dGPU Name>`).
- **Sub-30ms WSL GPU Caching**: Persistent caching for discrete GPU queries in WSL2, reducing execution time from ~1.7s to under 30ms.

### Changed
- Stripped redundant integrated graphics marketing strings (`with Radeon Graphics`, `with Intel UHD Graphics`) from CPU model lines for a cleaner terminal silhouette.

## [0.2.5] - 2026-08-19

### Added
- Native WSL2 hybrid GPU detection (`GpuCollector`) resolving both integrated graphics (e.g. `Intel Iris Xe` / `AMD Radeon Graphics`) and discrete NVIDIA graphics (e.g. `NVIDIA GeForce RTX 4080` / `RTX 4090`) via the native Windows driver bridge without extra Linux drivers.

## [0.2.0] - 2026-08-19

### Added
- `--json` CLI flag for structured JSON output across all enabled modules without external dependencies.
- Native `pci.ids` database parser resolving PCI vendor/device hex pairs to human-readable graphics cards without spawning subprocesses.
- WSL2 hypervisor and host motherboard model identification in `HostCollector`.
- Package manager counting support for **Homebrew** (`Cellar`) and **Gentoo** (`/var/db/pkg`).
- Foreground solid color block glyphs (`███`) in `ColorsCollector` for consistent light/dark terminal rendering.
- Built-in ASCII logos for **NixOS**, **Kali Linux**, **FreeBSD**, **Slackware**, **Artix Linux**, and **Zorin OS**.

### Fixed
- Fixed GPU detection prioritizing motherboard ACPI DMI slot labels (e.g. `Onboard - Video` on ASUS/Dell laptops) over actual graphics processor model names.

## [0.1.0] - 2026-08-16

### Added

- Core system information fetch engine implemented in Rust without spawning shell subprocesses.
- System metrics collectors:
  - **Title**: Username and hostname resolution via environment variables and `getpwuid`/`uname`.
  - **OS**: Linux distribution identification via `/etc/os-release` and `/usr/lib/os-release`.
  - **Host**: Hardware model and chassis parsing from `/sys/devices/virtual/dmi/id/` and device-tree.
  - **Kernel**: Release and architecture parsing from POSIX `libc::uname`.
  - **Uptime**: Accurate uptime calculation from `/proc/uptime` and `libc::sysinfo`.
  - **Packages**: Direct file-based package counting for Debian (`dpkg/status`), Arch (`pacman/local`), Red Hat (RPM database), Alpine (`apk`), Flatpak, and Snap.
  - **Shell**: Current shell process detection and version extraction via `/proc/<pid>/status` and `$SHELL`.
  - **Terminal**: Active terminal emulator detection via environment variables (`TERM_PROGRAM`, Alacritty, Kitty, Konsole, Foot) and process tree walking.
  - **Desktop / WM**: Desktop environment and window manager detection via `$XDG_CURRENT_DESKTOP`, Wayland socket signatures, and process scans.
  - **CPU**: Multi-socket, core count, and model name parsing from `/proc/cpuinfo` with vendor string sanitization.
  - **GPU**: Direct PCI sysfs scan (`/sys/bus/pci/devices`) with vendor ID mapping and fallback detection.
  - **Memory**: Accurate memory consumption calculation (`MemTotal - MemAvailable`) from `/proc/meminfo`.
  - **Disk**: Mount point capacity and utilization querying using POSIX `libc::statvfs`.
  - **Colors**: 8-color terminal palette block rendering.
- Layout and rendering engine:
  - Dynamic side-by-side logo and metric alignment.
  - ANSI escape code stripping for accurate visible character width calculation.
  - Automatic vertical layout fallback for narrow terminal displays (< 60 columns).
  - Terminal color auto-detection respecting `NO_COLOR`, `CLICOLOR_FORCE`, and non-TTY stdout redirection.
- Built-in ASCII logos:
  - Ferris the Rust mascot.
  - Distribution art for Arch, Debian, Ubuntu, Linux Mint, Fedora, RHEL, Rocky Linux, AlmaLinux, EndeavourOS, Manjaro, openSUSE, Alpine, Gentoo, Void Linux, Pop!_OS, and generic Tux.
- Command-line interface (`clap` derive):
  - `-m, --modules`: Module selection and ordering.
  - `-d, --disable`: Selective module disabling.
  - `-l, --logo`: ASCII logo override by distribution name or alias.
  - `--no-logo`: Logo suppression.
  - `--no-color`: ANSI color disabling.
  - `--disk-path`: Target path selection for disk metrics.
  - `--list-modules`: Available module enumeration.
- Comprehensive test suite:
  - Unit tests covering parsing logic across edge cases and malformed files.
  - Integration tests with synthetic procfs and sysfs fixtures for 15+ Linux distributions.
  - CLI flag combination tests using `assert_cmd`.
- Packaging and CI infrastructure:
  - GitHub Actions CI workflow for formatting, clippy, unit/integration testing, and release builds.
  - Release workflow building standalone GNU and Musl binaries, Debian (`.deb`), Red Hat (`.rpm`), Arch Linux (`.pkg.tar.zst`), and Android / Termux (`.deb` & ARM64 binary) packages with SHA256 checksums.
  - Arch Linux `PKGBUILD` and Debian packaging specifications.
