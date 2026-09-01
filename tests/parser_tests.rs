use kkfetch::modules::cpu::{clean_cpu_model, parse_cpu_info};
use kkfetch::modules::gpu::parse_lspci_mm_output;
use kkfetch::modules::memory::{format_memory, parse_meminfo};
use kkfetch::modules::os::parse_os_release;
use kkfetch::modules::packages::parse_dpkg_status;
use kkfetch::modules::uptime::{format_uptime, parse_uptime};
use kkfetch::modules::{ModuleId, ModuleOutput};
use kkfetch::output::formatter::{render_layout, visible_width};
use kkfetch::output::logo::match_logo;

// --- Distribution Family Tests ---

#[test]
fn test_fixture_debian_12() {
    let content = include_str!("fixtures/os_release/debian_12.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Debian GNU/Linux 12 (bookworm)");
    assert_eq!(info.distro_id, "debian");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "debian");
}

#[test]
fn test_fixture_ubuntu_24_04() {
    let content = include_str!("fixtures/os_release/ubuntu_24_04.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Ubuntu 24.04 LTS");
    assert_eq!(info.distro_id, "ubuntu");
    assert_eq!(info.distro_like, vec!["debian"]);
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "ubuntu");
}

#[test]
fn test_fixture_linux_mint_21() {
    let content = include_str!("fixtures/os_release/linux_mint_21.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Linux Mint 21.3");
    assert_eq!(info.distro_id, "linuxmint");
    assert_eq!(info.distro_like, vec!["ubuntu", "debian"]);
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "linuxmint");
}

#[test]
fn test_fixture_pop_os_22_04() {
    let content = include_str!("fixtures/os_release/pop_os_22_04.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Pop!_OS 22.04 LTS");
    assert_eq!(info.distro_id, "pop");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "pop");
}

#[test]
fn test_fixture_fedora_40() {
    let content = include_str!("fixtures/os_release/fedora_40.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Fedora Linux 40 (Workstation Edition)");
    assert_eq!(info.distro_id, "fedora");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "fedora");
}

#[test]
fn test_fixture_rhel_9() {
    let content = include_str!("fixtures/os_release/rhel_9.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Red Hat Enterprise Linux 9.3 (Plow)");
    assert_eq!(info.distro_id, "rhel");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "rhel");
}

#[test]
fn test_fixture_rocky_9() {
    let content = include_str!("fixtures/os_release/rocky_9.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Rocky Linux 9.4 (Blue Onyx)");
    assert_eq!(info.distro_id, "rocky");
    assert_eq!(info.distro_like, vec!["rhel", "centos", "fedora"]);
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "rocky");
}

#[test]
fn test_fixture_almalinux_9() {
    let content = include_str!("fixtures/os_release/almalinux_9.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "AlmaLinux 9.4 (Seafoam Ocelot)");
    assert_eq!(info.distro_id, "almalinux");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "almalinux");
}

#[test]
fn test_fixture_centos_stream_9() {
    let content = include_str!("fixtures/os_release/centos_stream_9.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "CentOS Stream 9");
    assert_eq!(info.distro_id, "centos");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "rhel");
}

#[test]
fn test_fixture_arch_rolling() {
    let content = include_str!("fixtures/os_release/arch_rolling.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Arch Linux");
    assert_eq!(info.distro_id, "arch");
    assert!(info.distro_like.is_empty());
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "arch");
}

#[test]
fn test_fixture_endeavouros_rolling() {
    let content = include_str!("fixtures/os_release/endeavouros_rolling.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "EndeavourOS");
    assert_eq!(info.distro_id, "endeavouros");
    assert_eq!(info.distro_like, vec!["arch"]);
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "endeavouros");
}

#[test]
fn test_fixture_manjaro_23() {
    let content = include_str!("fixtures/os_release/manjaro_23.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Manjaro Linux");
    assert_eq!(info.distro_id, "manjaro");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "manjaro");
}

#[test]
fn test_fixture_alpine_3_19() {
    let content = include_str!("fixtures/os_release/alpine_3_19.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Alpine Linux v3.19");
    assert_eq!(info.distro_id, "alpine");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "alpine");
}

#[test]
fn test_fixture_gentoo_rolling() {
    let content = include_str!("fixtures/os_release/gentoo_rolling.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Gentoo Linux");
    assert_eq!(info.distro_id, "gentoo");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "gentoo");
}

#[test]
fn test_fixture_void_rolling() {
    let content = include_str!("fixtures/os_release/void_rolling.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "Void Linux");
    assert_eq!(info.distro_id, "void");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "void");
}

#[test]
fn test_fixture_opensuse_tumbleweed() {
    let content = include_str!("fixtures/os_release/opensuse_tumbleweed.txt");
    let info = parse_os_release(content);
    assert_eq!(info.display_name, "openSUSE Tumbleweed");
    assert_eq!(info.distro_id, "opensuse-tumbleweed");
    let logo = match_logo(None, &info.distro_id, &info.distro_like).unwrap();
    assert_eq!(logo.name, "opensuse");
}

#[test]
fn test_fixture_os_release_corrupted_and_unquoted() {
    let corrupted = include_str!("fixtures/os_release/corrupted_no_equals.txt");
    let corrupt_info = parse_os_release(corrupted);
    assert_eq!(corrupt_info.display_name, "Linux");

    let unquoted = include_str!("fixtures/os_release/unquoted_values.txt");
    let unquoted_info = parse_os_release(unquoted);
    assert_eq!(unquoted_info.display_name, "Custom Arch Linux 2024");
}

// --- CPU Fixtures ---

#[test]
fn test_fixture_cpu_intel() {
    let content = include_str!("fixtures/cpuinfo/intel_i7_10750h.txt");
    let info = parse_cpu_info(content).expect("Failed to parse intel cpuinfo");
    assert_eq!(info.cores, 2);
    assert_eq!(info.sockets, 1);
    assert_eq!(clean_cpu_model(&info.model), "Intel Core i7-10750H");
}

#[test]
fn test_fixture_cpu_amd() {
    let content = include_str!("fixtures/cpuinfo/amd_ryzen_7700x.txt");
    let info = parse_cpu_info(content).expect("Failed to parse amd cpuinfo");
    assert_eq!(clean_cpu_model(&info.model), "AMD Ryzen 7 7700X");
}

#[test]
fn test_fixture_cpu_arm_raspberry_pi() {
    let content = include_str!("fixtures/cpuinfo/arm64_raspberry_pi.txt");
    let info = parse_cpu_info(content).expect("Failed to parse rpi cpuinfo");
    assert_eq!(info.cores, 2);
    assert_eq!(info.model, "BCM2835");
}

#[test]
fn test_fixture_cpu_dual_socket_xeon() {
    let content = include_str!("fixtures/cpuinfo/dual_socket_xeon.txt");
    let info = parse_cpu_info(content).expect("Failed to parse dual socket xeon");
    assert_eq!(info.cores, 2);
    assert_eq!(info.sockets, 2);
    assert_eq!(clean_cpu_model(&info.model), "Intel Xeon Gold 6248R");
}

#[test]
fn test_fixture_cpu_amd_epyc() {
    let content = include_str!("fixtures/cpuinfo/amd_epyc_server.txt");
    let info = parse_cpu_info(content).expect("Failed to parse amd epyc");
    assert_eq!(info.cores, 2);
    assert_eq!(clean_cpu_model(&info.model), "AMD EPYC 7763");
}

#[test]
fn test_fixture_cpu_riscv() {
    let content = include_str!("fixtures/cpuinfo/riscv64_qemu.txt");
    let info = parse_cpu_info(content).expect("Failed to parse riscv cpuinfo");
    assert_eq!(info.cores, 2);
    assert_eq!(info.model, "rv64imafdc (QEMU RISC-V)");
}

#[test]
fn test_fixture_cpu_power9() {
    let content = include_str!("fixtures/cpuinfo/power9_ibm.txt");
    let info = parse_cpu_info(content).expect("Failed to parse power9 cpuinfo");
    assert_eq!(info.cores, 2);
    assert_eq!(info.model, "POWER9, altivec supported");
}

#[test]
fn test_fixture_cpu_empty_and_malformed() {
    let empty = include_str!("fixtures/cpuinfo/empty_cpuinfo.txt");
    assert_eq!(parse_cpu_info(empty), None);

    let malformed = include_str!("fixtures/cpuinfo/malformed_cpuinfo.txt");
    assert_eq!(parse_cpu_info(malformed), None);
}

// --- Memory Fixtures ---

#[test]
fn test_fixture_meminfo_16gb() {
    let content = include_str!("fixtures/meminfo/standard_16gb.txt");
    let info = parse_meminfo(content).expect("Failed to parse meminfo");
    assert_eq!(info.total_kb, 16281600);
    assert_eq!(info.used_kb, 16281600 - 11550000);
    assert_eq!(info.percent, 29);
    let s = format_memory(&info);
    assert_eq!(s, "4.51 GiB / 15.53 GiB (29%)");
}

#[test]
fn test_fixture_meminfo_512mb() {
    let content = include_str!("fixtures/meminfo/low_memory_512mb.txt");
    let info = parse_meminfo(content).expect("Failed to parse 512mb meminfo");
    assert_eq!(info.total_kb, 524288);
    let s = format_memory(&info);
    assert!(s.contains("MiB"));
    assert_eq!(s, "256 MiB / 512 MiB (50%)");
}

#[test]
fn test_fixture_meminfo_128gb() {
    let content = include_str!("fixtures/meminfo/large_memory_128gb.txt");
    let info = parse_meminfo(content).expect("Failed to parse 128gb meminfo");
    assert_eq!(info.total_kb, 131828736);
    let s = format_memory(&info);
    assert!(s.contains("GiB"));
}

#[test]
fn test_fixture_meminfo_legacy_and_corrupted() {
    let legacy = include_str!("fixtures/meminfo/legacy_no_memavailable.txt");
    let info = parse_meminfo(legacy).expect("Failed to parse legacy meminfo");
    assert_eq!(info.total_kb, 8192000);
    assert!(info.used_kb > 0);

    let empty = include_str!("fixtures/meminfo/empty_meminfo.txt");
    assert_eq!(parse_meminfo(empty), None);

    let corrupted = include_str!("fixtures/meminfo/corrupted_meminfo.txt");
    assert_eq!(parse_meminfo(corrupted), None);
}

// --- Uptime Fixtures ---

#[test]
fn test_fixture_uptime_standard() {
    let content = include_str!("fixtures/uptime/standard.txt");
    let secs = parse_uptime(content).expect("Failed to parse uptime");
    assert_eq!(secs, 1978);
    let s = format_uptime(secs);
    assert_eq!(s, "32 mins");
}

#[test]
fn test_fixture_uptime_multi_day() {
    let content = include_str!("fixtures/uptime/multi_day_uptime.txt");
    let secs = parse_uptime(content).expect("Failed to parse multi-day uptime");
    let s = format_uptime(secs);
    assert_eq!(s, "10 days, 4 hours, 5 mins");
}

#[test]
fn test_fixture_uptime_single_day_single_hour() {
    let content = include_str!("fixtures/uptime/single_day_single_hour.txt");
    let secs = parse_uptime(content).expect("Failed to parse single day hour uptime");
    let s = format_uptime(secs);
    assert_eq!(s, "1 day, 1 hour, 1 min");
}

#[test]
fn test_fixture_uptime_zero_and_corrupted() {
    let zero = include_str!("fixtures/uptime/zero_uptime.txt");
    let secs = parse_uptime(zero).expect("Failed to parse zero uptime");
    assert_eq!(secs, 0);
    assert_eq!(format_uptime(secs), "0 mins");

    let negative = include_str!("fixtures/uptime/negative_or_corrupted.txt");
    assert_eq!(parse_uptime(negative), None);

    let empty = include_str!("fixtures/uptime/empty_uptime.txt");
    assert_eq!(parse_uptime(empty), None);
}

// --- DPKG Status Fixtures ---

#[test]
fn test_fixture_dpkg_status() {
    let content = include_str!("fixtures/dpkg/status_sample.txt");
    let count = parse_dpkg_status(content);
    assert_eq!(count, 3);
}

#[test]
fn test_fixture_dpkg_half_installed() {
    let content = include_str!("fixtures/dpkg/dpkg_half_installed_status.txt");
    let count = parse_dpkg_status(content);
    assert_eq!(count, 2); // only pkg1 (installed) and pkg6 (hold ok installed)
}

#[test]
fn test_fixture_dpkg_empty_and_corrupted() {
    let empty = include_str!("fixtures/dpkg/empty_dpkg_status.txt");
    assert_eq!(parse_dpkg_status(empty), 0);

    let corrupted = include_str!("fixtures/dpkg/corrupted_dpkg_status.txt");
    assert_eq!(parse_dpkg_status(corrupted), 0);
}

// --- GPU Fixtures & Layout Tests ---

#[test]
fn test_fixture_lspci_samples() {
    let content = include_str!("fixtures/gpu/lspci_samples.txt");
    let gpus = parse_lspci_mm_output(content);
    assert_eq!(gpus.len(), 2);
    assert_eq!(gpus[0], "Intel UHD Graphics");
    assert_eq!(gpus[1], "NVIDIA GeForce GTX 1650 Ti Mobile");
}

#[test]
fn test_visible_width_calculation() {
    assert_eq!(visible_width("\x1b[38;5;196mDebian\x1b[0m"), 6);
    assert_eq!(visible_width("Plain String"), 12);
}

#[test]
fn test_logo_resolution() {
    let debian = match_logo(None, "debian", &[]).unwrap();
    assert_eq!(debian.name, "debian");

    let rhel_fallback = match_logo(None, "alma", &["rhel".to_string()]).unwrap();
    assert_eq!(rhel_fallback.name, "almalinux");

    let win11 = match_logo(None, "windows11", &[]).unwrap();
    assert_eq!(win11.name, "windows11");

    let win10 = match_logo(None, "windows 10", &[]).unwrap();
    assert_eq!(win10.name, "windows10");

    let win7 = match_logo(None, "windows 7", &[]).unwrap();
    assert_eq!(win7.name, "windows7");
}

#[test]
fn test_render_layout_long_values_and_narrow_screen() {
    let logo = match_logo(Some("ferris"), "linux", &[]);
    let outputs = vec![
        ModuleOutput {
            id: ModuleId::Gpu,
            label: "GPU".to_string(),
            value: "NVIDIA GeForce RTX 4090, Intel Raptor Lake-S GT1 [UHD Graphics 770], VirtIO GPU Display Controller".to_string(),
            custom_rendered: None,
        },
        ModuleOutput {
            id: ModuleId::Host,
            label: "Host".to_string(),
            value: "Supermicro SuperServer SYS-741GE-TNRT Dual Socket E-ATX Workstation".to_string(),
            custom_rendered: None,
        },
    ];

    // Narrow terminal (vertical stacking)
    let narrow = render_layout(logo, &outputs, 50, false);
    assert!(narrow.contains("GPU: NVIDIA GeForce RTX 4090"));
    assert!(narrow.contains("Host: Supermicro SuperServer"));

    // Wide terminal (side-by-side)
    let wide = render_layout(logo, &outputs, 120, false);
    assert!(wide.contains("GPU: NVIDIA GeForce RTX 4090"));
    assert!(wide.contains("Host: Supermicro SuperServer"));
}
