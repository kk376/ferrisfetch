# FerrisFetch

A fast, lightweight, zero-subprocess system information fetch tool written in Rust for Linux, Windows, and macOS.

```text
            .-/+oossssoo+\-.               kk376@MSI-Thin-A15
        ´:+ssssssssssssssssss+:`           ------------------
      -+ssssssssssssssssssyyssss+-         OS: Ubuntu 24.04.4 LTS x86_64
    .ossssssssssssssssssdMMMNysssso.       Host: Windows Subsystem for Linux - 2.7.12.0
   /ssssssssssshdmmNNmmyNMMMMhssssss\      Kernel: 6.18.33.2-microsoft-standard-WSL2
  +ssssssssshmydMMMMMMMNddddyssssssss+     Installed: 23 Jan 2026, 12:22 AM (211 days ago)
 /sssssssshNMMMyhhyyyyhmNMMMNhssssssss\    Uptime: 3 hours, 14 mins
.ssssssssdMMMNhsssssssssshNMMMdssssssss.   Packages: 1197 (dpkg), 2 (cargo), 1 (npm)
+sssshhhyNMMNyssssssssssssyNMMMysssssss+   Shell: zsh 5.9
ossyNMMMNyMMhsssssssssssssshmmmhssssssso   Display: 1920x1080 @ 60Hz
ossyNMMMNyMMhsssssssssssssshmmmhssssssso   WM: WSLg 1.0.73.2 (Wayland)
+sssshhhyNMMNyssssssssssssyNMMMysssssss+   Terminal: Windows Terminal
.ssssssssdMMMNhsssssssssshNMMMdssssssss.   CPU: AMD Ryzen 5 7535HS (4) @ 3.294GHz
 \sssssssshNMMMyhhyyyyhdNMMMNhssssssss/    GPU0: AMD Radeon 660M (512 MiB) @ 1.900GHz [Integrated]
  +sssssssssdmydMMMMMMMMddddyssssssss+     GPU1: NVIDIA GeForce RTX 2050 (4 GiB) @ 2.100GHz [Discrete]
   \ssssssssssshdmNNNNmyNMMMMhssssss/      Memory: 1.31 GiB / 3.82 GiB (34%)
    .ossssssssssssssssssdMMMNysssso.       Swap: 0.00 GiB / 2.00 GiB (0%)
      -+sssssssssssssssssyyyssss+-         Disk0: (/) 21.9 GiB / 1006.9 GiB (2%) - ext4
        `:+ssssssssssssssssss+:`           Disk1: (C) 223.5 GiB / 475.9 GiB (47%) - ntfs
            .-\+oossssoo+/-.               Disk2: (D) 452.5 GiB / 931.5 GiB (49%) - ntfs
                                           Battery: 96% [AC Connected]
                                           Local IP: 172.30.193.167
                                           Theme: Adwaita [GTK/GNOME]
                                           Icons: Adwaita [GTK/GNOME]
```

---

## Why FerrisFetch?

Most fetch tools either spawn multiple shell child processes (`neofetch`) or dynamically link heavy C runtime libraries (`fastfetch`). FerrisFetch is built with a different design philosophy:

* **Sub-3ms Latency**: Queries virtual filesystems (`/proc`, `/sys`), POSIX syscalls, and Win32 APIs directly with zero child process spawning (`fork`/`execve`). In statistical benchmarks, it is **1.89x faster than Fastfetch** on raw data collection.
* **Native OS Install Date**: Probes root filesystem creation timestamp (`statx` birth time) and installer logs, showing exact installation date and relative age (`211 days ago`).
* **First-Class WSL2 & Windows Support**: Normalizes virtualized 9P and DrvFS network mounts (`/mnt/c`, `/mnt/d`) to native **NTFS** labels, discovers dual integrated and discrete GPUs, and detects WSLg displays.
* **Standalone Static Binary**: Zero libc runtime dependencies when using the musl build. Drop the binary into any Linux system and it runs.

---

## Benchmarks

Benchmarked against Fastfetch using [`hyperfine`](https://github.com/sharkdp/hyperfine) across **500+ iterations** on Ubuntu 24.04 (WSL2, AMD Ryzen 5 7535HS).

To eliminate terminal rendering and ANSI formatting differences, both tools were benchmarked using their machine-readable JSON output mode with shell process overhead disabled (`--shell=none`):

```bash
hyperfine --shell=none --warmup 50 --min-runs 500 \
  'fastfetch --format json' \
  'ferrisfetch --json'
```

### Results

| Command | Mean Runtime | Min Latency | Max Latency | User CPU Time | System Syscall Time | Relative Speedup |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| `fastfetch --format json` | `8.2 ms ± 3.4 ms` | `4.6 ms` | `26.2 ms` | `3.4 ms` | `4.8 ms` | `1.00` (Baseline) |
| `ferrisfetch --json` | **`4.3 ms ± 3.6 ms`** | **`1.8 ms`** | `29.7 ms` | **`1.6 ms`** | **`2.7 ms`** | **1.89 ± 1.63x faster** |

*FerrisFetch achieves lower CPU time and syscall overhead by reading `/proc` and `sysfs` directly in Rust, executing active module collectors concurrently in parallel using `std::thread::scope`, and compiling with Fat Link-Time Optimization (LTO).*

---

## Supported Operating Systems & Logos

FerrisFetch includes high-contrast ASCII art logos with distro brand signature colors for **26 operating systems and distributions**:

| Family / Ecosystem | Supported Distributions & Targets |
| :--- | :--- |
| **Debian / Ubuntu Family** | Ubuntu, Debian, Linux Mint, Pop!_OS (4) |
| **Red Hat Family** | Fedora, RHEL, Rocky Linux, AlmaLinux, CentOS Stream (5) |
| **Arch Family** | Arch Linux, EndeavourOS, Manjaro, Artix Linux (4) |
| **Independent Linux** | Alpine Linux, Gentoo Linux, Void Linux, openSUSE, NixOS (5) |
| **BSD Family** | FreeBSD, OpenBSD, NetBSD (3) |
| **Windows** | Windows 11, Windows 10 (native Win32 x86_64) (2) |
| **Android / Mobile** | Android (via Termux aarch64 & x86_64) (1) |
| **Mascots & Generic** | Ferris the Rust Crab (`ferris`), Linux Penguin (`tux`) (2) |

---

## Installation

### Ubuntu / Debian / Linux Mint / Pop!_OS

**Via Personal Package Archive (PPA):**
```bash
sudo add-apt-repository -y ppa:kushagra376/ferrisfetch
sudo apt update && sudo apt install -y ferrisfetch
```

**Via Pre-built `.deb`:**
```bash
curl -LO https://github.com/kk376/ferrisfetch/releases/download/v0.9.9/ferrisfetch_0.9.9-1_amd64.deb
sudo dpkg -i ferrisfetch_0.9.9-1_amd64.deb
```

---

### Fedora / RHEL / Rocky Linux / AlmaLinux

**Via Fedora Copr:**
```bash
sudo dnf copr enable -y kk376/ferrisfetch
sudo dnf install -y ferrisfetch
```

---

### Arch Linux / Manjaro / EndeavourOS

**Via Pre-built Pacman Package:**
```bash
curl -LO https://github.com/kk376/ferrisfetch/releases/download/v0.9.9/ferrisfetch-0.9.9-1-x86_64.pkg.tar.zst
sudo pacman -U ferrisfetch-0.9.9-1-x86_64.pkg.tar.zst
```

---

### macOS / Linux (Homebrew Tap)

```bash
brew install kk376/tap/ferrisfetch
```

---

### Android (Termux)

```bash
curl -LO https://github.com/kk376/ferrisfetch/releases/download/v0.9.9/ferrisfetch_0.9.9-1_termux_aarch64.deb
dpkg -i ferrisfetch_0.9.9-1_termux_aarch64.deb
```

---

### Windows (PowerShell / WinGet)

*Note: WinGet manifest submission is currently pending review in `microsoft/winget-pkgs`. Once merged, installation will be available via `winget install ferrisfetch`.*

**Manual Download & Run via PowerShell:**

```powershell
# 1. Download
curl.exe -LO https://github.com/kk376/ferrisfetch/releases/download/v0.9.9/ferrisfetch-windows-x86_64.zip

# 2. Extract
tar.exe -xf ferrisfetch-windows-x86_64.zip

# 3. Run
.\ferrisfetch.exe
```

---

### Universal Standalone Binary (Any 64-bit Linux)

Statically linked with musl (zero external dependencies):

```bash
curl -LO https://github.com/kk376/ferrisfetch/releases/download/v0.9.9/ferrisfetch-linux-musl-x86_64
chmod +x ferrisfetch-linux-musl-x86_64
sudo mv ferrisfetch-linux-musl-x86_64 /usr/local/bin/ferrisfetch
```

---

### Build from Source

Requires Rust 1.75.0+ and `gcc`.

```bash
git clone https://github.com/kk376/ferrisfetch.git
cd ferrisfetch
cargo build --release
sudo cp target/release/ferrisfetch /usr/local/bin/
```

---

## CLI Options

| Flag / Option | Description |
| :--- | :--- |
| `-m, --modules <LIST>` | Select and order specific modules (e.g. `os,kernel,cpu,memory`) |
| `-d, --disable <LIST>` | Disable specific modules from output (e.g. `gpu,disk`) |
| `-l, --logo <NAME>` | Override ASCII logo (e.g. `arch`, `debian`, `ferris`, `ubuntu`, `fedora`, `tux`, `none`) |
| `--no-logo` | Suppress the ASCII logo and print only system telemetry |
| `--no-color` | Disable ANSI color escapes |
| `--disk-path <PATH>` | Target filesystem path for disk statistics (default: `/`) |
| `--list-modules` | Print all available information modules and exit |
| `--json` | Output system information in structured JSON format |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

### Examples

**Custom module ordering:**
```bash
ferrisfetch -m os,cpu,memory,disk
```

**JSON output for scripts and status bars:**
```bash
ferrisfetch --json
```

**Disable specific modules:**
```bash
ferrisfetch -d gpu,packages
```

**Override logo with the Ferris mascot:**
```bash
ferrisfetch --logo ferris
```

---

## Information Modules & Detection Strategies

| Module | Primary Source | Fallback Strategy |
| :--- | :--- | :--- |
| **Title** | `$USER` / `getpwuid` and `uname(2)` | `"user@localhost"` |
| **OS** | `/etc/os-release` and `/usr/lib/os-release` parsing | `/etc/debian_version`, `/etc/redhat-release`, `uname` |
| **Host** | `/sys/devices/virtual/dmi/id/product_name` and devicetree model | DMI board name or omitted |
| **Kernel** | POSIX `libc::uname` release and machine fields | None required |
| **Installed** | Root filesystem creation timestamp via `statx(2)` (`stx_btime`) | Distribution install log birth times |
| **Uptime** | Floating-point parse of `/proc/uptime` | `libc::sysinfo` uptime |
| **Packages** | Local DB scans: `/var/lib/dpkg/status`, `pacman/local`, RPM, APK, flatpak, snap, cargo, npm, pip | `dpkg-query`, `rpm -qa`, `xbps-query` |
| **Shell** | `/proc/<pid>/status` & `comm` ancestor inspection | `$SHELL` environment variable |
| **Display** | DRM sysfs modes, `xrandr`, and `wlr-randr` refresh rates | Omitted if headless |
| **Desktop** | `$XDG_CURRENT_DESKTOP`, desktop metadata files, session type | Omitted if headless |
| **WM** | Active window manager detection (Mutter, KWin, Sway, Hyprland, WSLg) | Process scan |
| **Terminal** | Environment signatures (`WT_SESSION`, `TERM_PROGRAM`), `/proc` process ancestry | `$TERM` variable |
| **CPU** | `/proc/cpuinfo` parsing (model, clean brand, sockets, core count, frequency) | Sanitized model string |
| **GPU** | Sysfs PCI class scan (`0x03xxxx`), local `pci.ids` lookup, VRAM calculation | `lspci -mm` query |
| **Memory** | `/proc/meminfo` active memory calculation (`MemTotal - MemAvailable`) | Traditional buffer/cache calculation |
| **Swap** | `/proc/meminfo` swap statistics and ZRAM algorithm detection | Omitted if swap is 0 |
| **Disk** | Sequential filesystem partition discovery via `statvfs` (NTFS mapping on WSL) | Target mount path |
| **Battery** | Direct `/sys/class/power_supply` capacity and charging status | Omitted if AC-only desktop |
| **Local IP** | POSIX `getifaddrs` active interface address enumeration | Omitted if offline |
| **Theme** | GTK 3/4 `settings.ini`, KDE `kdeglobals`, XFCE `xsettings.xml`, `$GTK_THEME` | Omitted if not configured |
| **Icons** | GTK 3/4 `settings.ini`, KDE `kdeglobals`, XFCE `xsettings.xml` | Omitted if not configured |

---

## Shell Completions

FerrisFetch includes completions for Bash, Zsh, and Fish:

### Bash
```bash
source completions/ferrisfetch.bash
# System-wide: sudo cp completions/ferrisfetch.bash /usr/share/bash-completion/completions/ferrisfetch
```

### Zsh
```zsh
# Add to ~/.zshrc before compinit:
fpath=(/path/to/ferrisfetch/completions $fpath)
autoload -Uz compinit && compinit
# System-wide: sudo cp completions/_ferrisfetch /usr/share/zsh/site-functions/_ferrisfetch
```

### Fish
```fish
cp completions/ferrisfetch.fish ~/.config/fish/completions/
# System-wide: sudo cp completions/ferrisfetch.fish /usr/share/fish/vendor_completions.d/
```

---

## Distribution Packaging

Package definitions and build specifications are organized in [`packaging/`](packaging/):

* **Arch Linux (AUR)**: [`packaging/arch/`](packaging/arch/) (`PKGBUILD`, `.SRCINFO`)
* **Debian / Ubuntu**: [`packaging/debian/`](packaging/debian/) (`control`, `rules`, `changelog`)
* **Fedora / RHEL (Copr)**: [`packaging/rpm/`](packaging/rpm/) (`ferrisfetch.spec`)
* **Alpine Linux**: [`packaging/alpine/`](packaging/alpine/) (`APKBUILD`)
* **Gentoo Linux**: [`packaging/gentoo/`](packaging/gentoo/) (`ferrisfetch-0.9.9.ebuild`)
* **Void Linux**: [`packaging/void/`](packaging/void/) (`template`)
* **Nix / NixOS**: [`packaging/nix/`](packaging/nix/) (`package.nix`)
* **Homebrew Tap**: [`packaging/homebrew/`](packaging/homebrew/) (`ferrisfetch.rb`)
* **Android (Termux)**: [`packaging/termux/`](packaging/termux/) (`build.sh`)
* **Windows (WinGet)**: [`packaging/winget/`](packaging/winget/) (YAML manifests)

---

## Development & Testing

```bash
# Run the full test suite (unit, integration, and CLI snapshot tests)
cargo test

# Run strict linter with zero warnings allowed
cargo clippy --all-targets --all-features -- -D warnings

# Verify code formatting conforms to Rust standards
cargo fmt --check
```

---

## Community Acknowledgements

Special thanks to community contributors for architectural recommendations:

* **[@Laynsb](https://github.com/Laynsb)**:
  * **System Installation Date Module (`Installed`)**: Suggested adding OS installation date detection via root filesystem `statx` birth time (`stx_btime`) with relative time deltas.
  * **Localized Installation Timestamps**: Suggested local timezone conversion for wall-clock consistency.
  * **Filesystem Type Detection (`Disk`)**: Suggested partition filesystem labeling.
  * **ZRAM Compression Algorithm Discovery (`Swap`)**: Suggested detecting active swap compression algorithms from `/sys/block/zram*/comp_algorithm`.

---

## Contributing & Security

* **Contributing**: Pull requests, feature ideas, and packaging recipes are welcome! Please review [CONTRIBUTING.md](CONTRIBUTING.md) for architectural guidelines, coding standards, and PR workflows.
* **Security Policy**: For reporting security vulnerabilities or policy questions, please refer to [SECURITY.md](SECURITY.md).
* **Changelog**: Complete release history across versions is tracked in [CHANGELOG.md](CHANGELOG.md).

---

## Credits & License

* **FerrisFetch** is open-source software licensed under the **[MIT License](LICENSE)**.
* **ASCII Art Outlines**: Distribution ASCII art boundary outlines are based on the classic art from **[Neofetch](https://github.com/dylanaraps/neofetch)** by Dylan Araps (also licensed under the **MIT License**, Copyright © 2016-2022 Dylan Araps), customized and enhanced in FerrisFetch with high-contrast white structural framing and distribution brand signature colors.
