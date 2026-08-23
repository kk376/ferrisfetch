%global debug_package %{nil}

Name:           ferrisfetch
Version:        0.9.10
Release:        1%{?dist}
Summary:        A fast, lightweight Linux, macOS, and Windows system information fetch tool written in Rust

License:        MIT
URL:            https://github.com/kk376/ferrisfetch
Source0:        https://github.com/kk376/ferrisfetch/archive/refs/tags/v%{version}.tar.gz#/%{name}-%{version}.tar.gz

BuildRequires:  cargo >= 1.75.0
BuildRequires:  rust >= 1.75.0
BuildRequires:  gcc

%description
FerrisFetch is a fast, zero-runtime-dependency CLI system information fetch tool
written in Rust, specifically designed for Linux distributions. It gathers system
metrics including OS release, kernel version, CPU, GPU, memory, disk usage,
package managers, desktop environment, uptime, and shell information, formatting
them cleanly alongside colorful ANSI distribution ASCII logos.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release

%check
cargo test --release

%install
# Install executable binary
install -Dpm 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install shell completions
install -Dpm 0644 completions/%{name}.bash %{buildroot}%{_datadir}/bash-completion/completions/%{name}
install -Dpm 0644 completions/_%{name} %{buildroot}%{_datadir}/zsh/site-functions/_%{name}
install -Dpm 0644 completions/%{name}.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/%{name}.fish

# Install documentation
install -Dpm 0644 README.md %{buildroot}%{_docdir}/%{name}/README.md

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_datadir}/bash-completion/completions/%{name}
%dir %{_datadir}/zsh/site-functions
%{_datadir}/zsh/site-functions/_%{name}
%dir %{_datadir}/fish/vendor_completions.d
%{_datadir}/fish/vendor_completions.d/%{name}.fish

%changelog
* Sun Aug 23 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.9.10-1
- Release version 0.9.10: Zero-subprocess Windows Toolhelp32 process snapshot shell and terminal detection

* Sun Aug 23 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.9.9-1
- Release version 0.9.9
- Preserve package checksum hashes in vendored crates

* Sun Aug 23 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.9.8-1
- Release version 0.9.8
- Fix vendored cargo checksums and offline paths

* Sun Aug 23 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.9.7-1
- Release version 0.9.7
- RPM MTIME package caching for sub-millisecond query latency
- DRM-first display connector parsing

* Sun Aug 23 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.9.6-1
- Release version 0.9.6
- WSL Host Version Discovery: Reports host WSL version on Host line
- Vendored cargo checksum fix for offline builds

* Sat Aug 22 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.9.5-1
- Release version 0.9.5
- WSLg Version Discovery: Probes and reports active WSLg version
- GPU Type Classification: Identifies and annotates [Integrated] vs [Discrete] GPUs

* Sat Aug 22 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.9.0-1
- Release version 0.9.0
- Enhanced ASCII Distro Art: High-contrast white outer framing with brand-colored inner emblems across all 26+ distributions
- WSL2 Storage Resolution: Normalized 9p/drvfs virtualization filesystem mappings to native NTFS for Windows drive mounts

* Sun Aug 16 2026 FerrisFetch Packaging Team <packaging@ferrisfetch.rs> - 0.1.0-1
- Initial RPM release for version 0.1.0
- Added modular system metric collectors
- Added multi-distro ANSI 256-color ASCII art
- Added Bash, Zsh, and Fish shell completions
