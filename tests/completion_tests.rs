use std::fs;
use std::path::Path;

#[test]
fn test_completion_files_exist_and_cover_all_flags() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("completions");
    let bash_file = base_dir.join("kkfetch.bash");
    let zsh_file = base_dir.join("_kkfetch");
    let fish_file = base_dir.join("kkfetch.fish");

    assert!(bash_file.is_file(), "Bash completion file missing");
    assert!(zsh_file.is_file(), "Zsh completion file missing");
    assert!(fish_file.is_file(), "Fish completion file missing");

    let bash_content = fs::read_to_string(&bash_file).unwrap();
    let zsh_content = fs::read_to_string(&zsh_file).unwrap();
    let fish_content = fs::read_to_string(&fish_file).unwrap();

    assert!(
        bash_content.contains("kkfetch"),
        "Bash completions missing kkfetch command name"
    );
    assert!(
        zsh_content.contains("kkfetch"),
        "Zsh completions missing kkfetch command name"
    );
    assert!(
        fish_content.contains("kkfetch"),
        "Fish completions missing kkfetch command name"
    );

    let required_flags = [
        "--modules",
        "--disable",
        "--logo",
        "--no-color",
        "--no-logo",
        "--list-modules",
        "--disk-path",
        "--json",
        "--help",
        "--version",
    ];

    let required_modules = [
        "title", "os", "host", "kernel", "uptime", "packages", "shell", "desktop", "terminal",
        "cpu", "gpu", "memory", "disk", "theme", "icons", "colors",
    ];

    let required_logos = [
        "ferris",
        "debian",
        "ubuntu",
        "linuxmint",
        "fedora",
        "arch",
        "rhel",
        "rocky",
        "almalinux",
        "endeavouros",
        "manjaro",
        "generic",
        "opensuse",
        "alpine",
        "gentoo",
        "void",
        "pop",
        "none",
    ];

    for flag in &required_flags {
        assert!(
            bash_content.contains(flag),
            "Bash completions missing flag: {}",
            flag
        );
        assert!(
            zsh_content.contains(flag),
            "Zsh completions missing flag: {}",
            flag
        );
        let flag_name = flag.trim_start_matches("--");
        assert!(
            fish_content.contains(flag) || fish_content.contains(&format!("-l {}", flag_name)),
            "Fish completions missing flag: {}",
            flag
        );
    }

    for module in &required_modules {
        assert!(
            bash_content.contains(module),
            "Bash completions missing module: {}",
            module
        );
        assert!(
            zsh_content.contains(module),
            "Zsh completions missing module: {}",
            module
        );
        assert!(
            fish_content.contains(module),
            "Fish completions missing module: {}",
            module
        );
    }

    for logo in &required_logos {
        assert!(
            bash_content.contains(logo),
            "Bash completions missing logo: {}",
            logo
        );
        assert!(
            zsh_content.contains(logo),
            "Zsh completions missing logo: {}",
            logo
        );
        assert!(
            fish_content.contains(logo),
            "Fish completions missing logo: {}",
            logo
        );
    }
}
