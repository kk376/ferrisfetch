use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "A fast, lightweight Linux system information fetch tool",
        ))
        .stdout(predicate::str::contains("--modules"))
        .stdout(predicate::str::contains("--list-modules"));
}

#[test]
fn test_list_modules_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.arg("--list-modules");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("os"))
        .stdout(predicate::str::contains("kernel"))
        .stdout(predicate::str::contains("cpu"))
        .stdout(predicate::str::contains("memory"))
        .stdout(predicate::str::contains("uptime"))
        .stdout(predicate::str::contains("shell"))
        .stdout(predicate::str::contains("disk"));
}

#[test]
fn test_no_color_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.arg("--no-color");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains("\x1b["));
}

#[test]
fn test_custom_modules_filtering() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args(["--no-color", "--no-logo", "-m", "os,kernel"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("OS:"));
    assert!(stdout.contains("Kernel:"));
    assert!(!stdout.contains("CPU:"));
    assert!(!stdout.contains("Memory:"));
}

#[test]
fn test_disable_module_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args([
        "--no-color",
        "--no-logo",
        "-m",
        "os,kernel,cpu",
        "-d",
        "cpu",
    ]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("OS:"));
    assert!(stdout.contains("Kernel:"));
    assert!(!stdout.contains("CPU:"));
}

#[test]
fn test_logo_override_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args(["--no-color", "--logo", "arch", "-m", "os"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("oooo"));
    assert!(stdout.contains("OS:"));
}

#[test]
fn test_no_logo_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args(["--no-color", "--no-logo", "-m", "os"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let expected_os = ferrisfetch::modules::os::detect_os().display_name;
    assert!(stdout.contains(&format!("OS: {}", expected_os)));
}

#[test]
fn test_disk_path_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args(["--no-color", "--no-logo", "-m", "disk", "--disk-path", "/"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Disk0:"));
    assert!(stdout.contains('%'));
}

#[test]
fn test_disk_path_invalid_does_not_panic() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args([
        "--no-color",
        "--no-logo",
        "-m",
        "disk",
        "--disk-path",
        "/path/that/does/not/exist_12345",
    ]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Disk module should cleanly produce no output rather than crashing
    assert_eq!(stdout.trim(), "");
}

#[test]
fn test_duplicate_and_invalid_module_args() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args([
        "--no-color",
        "--no-logo",
        "-m",
        "os,invalid_module_xyz,os,kernel,cpu,cpu",
    ]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // OS, Kernel, CPU should appear exactly once
    assert_eq!(stdout.matches("OS:").count(), 1);
    assert_eq!(stdout.matches("Kernel:").count(), 1);
    assert_eq!(stdout.matches("CPU:").count(), 1);
    assert!(!stdout.contains("invalid_module_xyz"));
}

#[test]
fn test_combine_modules_and_disable_flags() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args([
        "--no-color",
        "--no-logo",
        "-m",
        "os,kernel,cpu,memory,uptime",
        "-d",
        "cpu,uptime",
    ]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.contains("OS:"));
    assert!(stdout.contains("Kernel:"));
    assert!(stdout.contains("Memory:"));
    assert!(!stdout.contains("CPU:"));
    assert!(!stdout.contains("Uptime:"));
}

#[test]
fn test_installed_module_filtering() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args(["--no-color", "--no-logo", "-m", "installed"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Installed:"));
    assert!(!stdout.contains("OS:"));
}

#[test]
fn test_various_logo_overrides() {
    let logos = [
        ("debian", "met$$$$$gg"),
        ("ubuntu", "ssss"),
        ("arch", "oooo"),
        ("fedora", "cccc"),
        ("mint", "MMMM"),
        ("tux", "#####"),
        ("windows11", "llll"),
        ("windows10", "llll"),
        ("windows", "llll"),
        ("win11", "llll"),
        ("win10", "llll"),
    ];

    for (name, snippet) in logos {
        let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
        cmd.args(["--no-color", "--logo", name, "-m", "os"]);
        let assert = cmd.assert().success();
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        assert!(
            stdout.contains(snippet),
            "Logo override '{}' did not produce expected pattern '{}'",
            name,
            snippet
        );
    }
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "ferrisfetch {}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn test_json_output_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.arg("--json");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.starts_with('{'));
    assert!(stdout.trim().ends_with('}'));
    assert!(stdout.contains("\"os\":"));
    assert!(stdout.contains("\"kernel\":"));
}

#[test]
fn test_timings_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args([
        "--no-color",
        "--no-logo",
        "--timings",
        "-m",
        "os,kernel,cpu",
    ]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Module Execution Timings"));
    assert!(stdout.contains("os"));
    assert!(stdout.contains("kernel"));
    assert!(stdout.contains("cpu"));
    assert!(stdout.contains("Total Time"));
}

#[test]
fn test_timings_with_json_flag() {
    let mut cmd = Command::cargo_bin("ferrisfetch").unwrap();
    cmd.args(["--json", "--timings", "-m", "os,kernel"]);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stdout.starts_with('{'));
    assert!(stderr.contains("Module Execution Timings"));
    assert!(stderr.contains("os"));
    assert!(stderr.contains("kernel"));
}
