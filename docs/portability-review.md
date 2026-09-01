# KKFetch Portability and Test Coverage Review

## 1. Scope and Objective

This review documents the portability mechanisms, failure mode handling, and test fixtures added to KKFetch. KKFetch targets sub-5 millisecond execution on standard Linux distributions without mandatory external runtime dependencies. The tool must operate reliably across Debian, Red Hat, Arch, and independent distribution families, as well as virtualized environments and headless servers.

All file I/O operations and subprocess calls are non-blocking, safe against missing files or commands, and strictly local with zero network calls.

---

## 2. Distribution Coverage and Mechanisms

KKFetch resolves operating system metadata using standard systemd specifications with fallbacks for legacy and minimal distributions.

```
+-------------------------------------------------------------------------+
|                              detect_os()                                |
+-------------------------------------------------------------------------+
                                    |
            +-----------------------+-----------------------+
            |                                               |
            v                                               v
 1. Standard os-release                          2. Legacy release files
    - /etc/os-release                              - /etc/debian_version (Debian)
    - /usr/lib/os-release (stateless)              - /etc/redhat-release (RHEL/CentOS)
                                                   - /etc/arch-release   (Arch)
                                                   - /etc/gentoo-release (Gentoo)
                                                   - /etc/alpine-release (Alpine)
                                                            |
                                                            v
                                                 3. POSIX uname fallback
                                                    - libc::uname -> "Linux"
```

### Tested Distribution Families

| Distribution Family | Tested Distributions | Release Identifier (`ID`) | Logo Matching Mechanism |
|---|---|---|---|
| **Debian Family** | Debian 12 (Bookworm)<br>Ubuntu 24.04 LTS<br>Linux Mint 21.3<br>Pop!_OS 22.04 LTS | `debian`<br>`ubuntu`<br>`linuxmint`<br>`pop` | Direct logo matching; fallback to `debian` via `ID_LIKE` |
| **Red Hat Family** | Fedora 40<br>RHEL 9.3<br>Rocky Linux 9.4<br>AlmaLinux 9.4<br>CentOS Stream 9 | `fedora`<br>`rhel`<br>`rocky`<br>`almalinux`<br>`centos` | Direct logo matching; fallback to `rhel`/`fedora` via `ID_LIKE` |
| **Arch Family** | Arch Linux (rolling)<br>EndeavourOS (rolling)<br>Manjaro Linux 23 | `arch`<br>`endeavouros`<br>`manjaro` | Direct logo matching; fallback to `arch` via `ID_LIKE` |
| **Independent Distros** | Alpine Linux 3.19<br>Gentoo Linux<br>Void Linux<br>openSUSE Tumbleweed | `alpine`<br>`gentoo`<br>`void`<br>`opensuse-tumbleweed` | Direct logo matching; fallback to `generic` (Tux) or `ferris` |

### Malformed Input Handling
- **Empty `/etc/os-release`**: Returns fallback `OsInfo { display_name: "Linux", distro_id: "linux", distro_like: [] }`.
- **Corrupted lines (no `=` delimiter)**: Skipped without error or panic.
- **Unquoted values**: Parsed correctly without requiring surrounding double or single quotes.
- **Empty value fields (`NAME=""`)**: Ignored so fallback cascades to secondary fields.

---

## 3. Package Manager Detection

KKFetch uses direct file parsing for package managers that maintain plain-text metadata. Subprocesses are spawned only when necessary, and only after verifying database existence on disk.

```
+-------------------------------------------------------------------------------+
|                            get_packages_summary()                             |
+-------------------------------------------------------------------------------+
        |               |               |               |               |
        v               v               v               v               v
     [dpkg]         [pacman]          [rpm]       [apk / xbps]  [flatpak / snap]
 /var/lib/dpkg/  /var/lib/pacman/  Check DB first   Direct file   Directory scan
    status            local/       -> rpm -qa       or query      sys & user paths
```

### Supported Package Managers

1. **dpkg (Debian, Ubuntu, Mint, Pop!_OS)**
   - Mechanism: Direct read of `/var/lib/dpkg/status`. Counts lines matching `Status: * installed`.
   - Filtering: Ignores `deinstall ok config-files`, `half-installed`, `half-configured`, and `unpacked`. Includes `hold ok installed`.
   - Fallback: Spawns `dpkg-query -f '${binary:Package}\n' -W` if the status file is inaccessible.
   - Latency: Less than 1 ms for standard databases via memory buffer scanning.

2. **pacman (Arch, EndeavourOS, Manjaro)**
   - Mechanism: Direct directory read of `/var/lib/pacman/local/`.
   - Filtering: Counts directory entries while skipping hidden directories (`.*`) and metadata files like `ALPM_DB_VERSION`.
   - Latency: Less than 0.5 ms via single directory listing.

3. **rpm (Fedora, RHEL, Rocky, Alma, CentOS)**
   - Mechanism: Checks database existence (`/var/lib/rpm/Packages`, `/var/lib/rpm/rpmdb.sqlite`, `/usr/lib/sysimage/rpm/Packages`, `/usr/lib/sysimage/rpm/rpmdb.sqlite`) before executing `rpm -qa`.
   - Safety: Avoids spawning `rpm` on Debian or Arch systems where `rpm` is absent or unused.

4. **apk (Alpine Linux)**
   - Mechanism: Direct read of `/lib/apk/db/installed`. Counts lines starting with `P:`.

5. **xbps (Void Linux)**
   - Mechanism: Checks `/var/db/xbps/pkgdb*` and runs `xbps-query -l`.

6. **Flatpak & Snap**
   - Flatpak: Scans `/var/lib/flatpak/app` and `~/.local/share/flatpak/app`.
   - Snap: Scans `/var/lib/snapd/snaps/*.snap` and `/snap/*` excluding `bin` and `README`.

---

## 4. Hardware and Virtualization Probing

### CPU Detection
- **Architectures**: x86_64, aarch64, armv7l, riscv64, ppc64le.
- **Keys Handled**: `model name`, `Hardware`, `cpu model`, `cpu` (PowerPC), `Model` (devicetree), `processor`, `physical id`.
- **Physical Sockets**: Aggregates distinct `physical id` values from `/proc/cpuinfo`. Sockets count is displayed when greater than 1 (e.g. `Intel Xeon Gold 6248R (2 sockets, 48 cores)`).
- **Model String Sanitization**: Strips redundant marketing noise (`(R)`, `(TM)`, `CPU`, `Processor`, `Dual-Core` up to `128-Core`) and clock speed suffixes (`@ 2.60GHz`).
- **Failure Modes**: Empty or corrupted `/proc/cpuinfo` returns `None`.

### Memory Detection
- **Linux 3.14+**: Calculates used memory as `MemTotal - MemAvailable`.
- **Pre-3.14 Fallback**: Calculates non-used memory as `MemFree + Buffers + Cached + SReclaimable - Shmem`, subtracting from `MemTotal`.
- **Bounds Checking**: Used memory is clamped to `MemTotal`, and utilization percentage is clamped to 100%.
- **Formatting**: Values >= 1 GiB display in GiB with two decimals; values < 1 GiB display in MiB.

### GPU Detection
- **PCI Class Filtering**: Scans `/sys/bus/pci/devices/` for PCI display classes `0x0300` (VGA), `0x0302` (3D Controller), and `0x0380` (Display).
- **Vendor Mapping**: Resolves raw vendor IDs to clean names:
  - `0x10de`: NVIDIA
  - `0x1002`: AMD
  - `0x8086`: Intel
  - `0x1af4`: VirtIO GPU
  - `0x1414`: Microsoft Direct3D (Hyper-V / WSL2)
  - `0x15ad`: VMware SVGA
  - `0x80ee`: VirtualBox Graphics
  - `0x1b36`: Red Hat QXL
  - `0x1a03`: ASPEED Graphics
  - `0x13d7`: Broadcom VideoCore
  - `0x5143`: Qualcomm Adreno
- **Multi-GPU Setups**: Correctly detects and joins hybrid GPU configurations (e.g. `Intel UHD Graphics, NVIDIA GeForce GTX 1650 Ti`).
- **lspci Fallback**: When sysfs yields unmapped numeric device IDs, parses `lspci -mm` output and strips redundant vendor strings (`Corporation`, `Inc.`, `[AMD/ATI]`).

### Disk Usage
- **Mechanism**: Calls POSIX `libc::statvfs` directly.
- **Calculations**: Computes total, free, and used capacity using `f_frsize` (or `f_bsize`) and block counts.
- **Safety**: Rejects invalid paths containing null bytes or non-existent mount points, returning `None` instead of panicking.

---

## 5. Desktop, Window Manager, and Terminal Detection

### Desktop Environment and Window Manager
- **Desktop Environment**: Reads `XDG_CURRENT_DESKTOP` (handles colon-separated values like `ubuntu:GNOME` -> `GNOME`) and `DESKTOP_SESSION`.
- **Wayland Window Managers**: Inspects environment signatures for Sway (`SWAYSOCK`), Hyprland (`HYPRLAND_INSTANCE_SIGNATURE`), Wayfire (`WAYFIRE_CONFIG_FILE`), River (`RIVER_SOCKET`), and labwc (`LABWC_PID`).
- **X11 Process Fallback**: Scans `/proc/<pid>/comm` for running window managers (`i3`, `bspwm`, `awesome`, `dwm`, `openbox`, `xmonad`, `qtile`, `mutter`, `kwin`, `xfwm4`).
- **Headless Detection**: Returns `None` when neither a DE, graphical WM, nor valid display session is present. `XDG_SESSION_TYPE=tty` is treated as headless.

### Shell and Terminal
- **Shell**: Traverses `/proc/<pid>/status` parent IDs up to 5 levels to identify the active interactive shell process (`bash`, `zsh`, `fish`, `sh`, `dash`, `nu`, etc.), falling back to `$SHELL`. Extracts shell versions from `$BASH_VERSION`, `$ZSH_VERSION`, or `$FISH_VERSION`.
- **Terminal Emulator**: Checks `$TERM_PROGRAM` + `$TERM_PROGRAM_VERSION`, dedicated environment signatures (`ALACRITTY_SOCKET`, `KITTY_PID`, `KONSOLE_VERSION`, `WT_SESSION`, `FOOT_PID`, `XTERM_VERSION`, `WEZTERM_PANE`), and process ancestry up to 8 levels.

---

## 6. CLI Flags and Formatting Verification

### CLI Behavior
- **Module Filtering (`-m`)**: Accepts comma-separated module names, deduplicating entries while preserving order.
- **Module Disabling (`-d`)**: Removes specified modules from the active set.
- **Invalid Module Names**: Silently ignored without crashing.
- **Color Control**: Disabled when `--no-color` is passed, `NO_COLOR` is set, `TERM=dumb`, or stdout is redirected to a non-TTY pipe.
- **Custom Disk Path (`--disk-path`)**: Queries any mount point or directory. Unreadable paths cleanly omit the disk line.

### Layout Engine
- **Visible Width**: Calculates visible printable column width by stripping ANSI escape sequences.
- **Responsive Layout**:
  - Terminal width >= 60 columns: Two-column side-by-side layout with padded alignment.
  - Terminal width < 60 columns: Vertical stacked layout (logo on top, blank line, module details below).
  - Logo disabled (`--no-logo` or `-l none`): Emits module rows directly without indentation.
  - Long values: Lines render completely without truncation.

---

## 7. Test Results Summary

The test suite contains 127 automated tests covering unit functions, distribution fixtures, and CLI binary execution.

```
--------------------------------------------------------------------------------
Test Category           Test Count   Status   Description
--------------------------------------------------------------------------------
Unit Tests (lib.rs)             75   PASS     Module parsers, formatters, safety
CLI Integration Tests           12   PASS     Binary argument execution and flags
Parser & Fixture Tests          40   PASS     Distro, CPU, memory, uptime fixtures
--------------------------------------------------------------------------------
Total                          127   PASS     Zero failures, zero warnings
--------------------------------------------------------------------------------
```

All tests pass cleanly under `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings`.
