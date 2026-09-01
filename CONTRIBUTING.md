# Contributing to KKFetch

Thank you for contributing to KKFetch. This guide covers local development, testing, adding new system modules, and contributing distribution logos.

## Maintainers

KKFetch is maintained by **Kushagra Kumar (kk376)**.

## Prerequisites

- Rust 1.75.0 or later (standard toolchain via `rustup`)
- Standard C library headers (`libc`)
- Optional packaging utilities: `cargo-deb`, `cargo-generate-rpm`, `zstd`

## Development Workflow

### Build from source

```bash
git clone https://github.com/kk376/kkfetch.git
cd kkfetch
cargo build
```

Run KKFetch locally:

```bash
cargo run -- --logo ferris
```

### Running tests and checks

All pull requests must pass formatting, linter, and test checks without warnings.

1. **Check formatting:**
   ```bash
   cargo fmt --check
   ```
   Apply automatic formatting with `cargo fmt`.

2. **Run Clippy linter:**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Run the test suite:**
   ```bash
   cargo test --all-targets --all-features
   ```

## Adding a New Module

Modules collect and format a specific piece of system information.

### Step 1: Define the module file

Create `src/modules/<module_name>.rs` and implement the `Collector` trait:

```rust
use crate::context::FetchContext;
use crate::modules::{Collector, ModuleId, ModuleOutput};

pub struct MyModuleCollector;

impl Collector for MyModuleCollector {
    fn id(&self) -> ModuleId {
        ModuleId::MyModule
    }

    fn collect(&self, _ctx: &FetchContext) -> Option<ModuleOutput> {
        let value = detect_my_metric()?;
        Some(ModuleOutput {
            id: self.id(),
            label: "My Metric".to_string(),
            value,
            custom_rendered: None,
        })
    }
}

pub fn detect_my_metric() -> Option<String> {
    // Read from /proc or /sys directly without spawning child processes.
    // Return None if the data is unavailable or unreadable.
    std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .map(|s| format!("{}%", s.trim()))
}
```

### Step 2: Register in `src/modules/mod.rs`

1. Add `pub mod <module_name>;`
2. Add the variant to `ModuleId` enum.
3. Update `ModuleId::all()`, `ModuleId::as_str()`, and `FromStr` implementations.
4. Register the collector instance inside `ModuleRegistry::new()`.

### Step 3: Add unit tests

Include unit tests in `src/modules/<module_name>.rs` to test edge cases, missing files, or corrupted string parsing. Add mock fixtures in `tests/parser_tests.rs` if testing against synthetic procfs/sysfs data.

## Adding a Distribution Logo

KKFetch contains compact ASCII art logos for Linux distributions.

### Logo Guidelines

- **Dimensions**: Keep logos compact (typically 4 to 8 lines high, under 25 columns wide) so they fit comfortably next to module text on 80-column terminals.
- **Escape sequences**: Use raw strings without embedded ANSI codes in `raw_lines`. Colors are applied dynamically via `primary_color`.

### Step 1: Add logo entry in `src/output/logo.rs`

Append the new `Logo` definition to `ALL_LOGOS`:

```rust
Logo {
    name: "mydistro",
    raw_lines: &[
        "  /\\___/\\",
        " (  o o  )",
        " (  =^=  )",
        "  (---)",
    ],
    primary_color: "\x1b[38;5;39m",
},
```

### Step 2: Add matching keys in `find_logo_by_key`

Map distro IDs and aliases (matching `/etc/os-release` `ID` and `ID_LIKE` values):

```rust
"mydistro" | "mydistro-linux" => logos.iter().find(|l| l.name == "mydistro"),
```

### Step 3: Add a test case

Add test coverage in `src/output/logo.rs` verifying that `match_logo` resolves your new identifier.

## Core Architectural Guidelines

- **Zero shell subprocesses**: Always query metrics via `/proc`, `/sys`, environment variables, or standard `libc` interfaces. Spawning subprocesses (`sh`, `bash`, or external CLI binaries) slows down execution and introduces unnecessary runtime dependencies.
- **No panics**: System metrics must never cause a crash. Use `Option`, `Result`, and defensive parsing. If a file does not exist or has an unexpected format, return `None` or an empty string so the module can be omitted gracefully.
- **Minimal allocations**: Avoid redundant string clones and intermediate buffers in tight parsing loops.
- **ANSI width awareness**: If modifying layouts, calculate visible terminal widths by stripping ANSI sequences rather than using raw byte lengths.

## Pull Request Process

1. Fork the repository and create a descriptive branch: `git checkout -b feature/battery-module` or `git checkout -b fix/arch-logo-alignment`.
2. Ensure `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` pass cleanly.
3. Submit a pull request filling in the pull request template with a brief description of the change and how it was verified.
