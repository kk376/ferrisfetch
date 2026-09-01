#![warn(clippy::undocumented_unsafe_blocks)]
use clap::Parser;
use kkfetch::cli::Cli;
use kkfetch::context::FetchContext;
use kkfetch::modules::{ModuleId, ModuleRegistry};
use kkfetch::output::formatter::{render_json, render_layout};
use kkfetch::output::logo::match_logo;

fn main() {
    let cli = Cli::parse();

    // Early exit for shell completion scripts or discovery tooling
    if cli.list_modules {
        for module in ModuleId::all() {
            println!("{}", module.as_str());
        }
        return;
    }

    // Initialize execution context once to share terminal dimensions and OS release metadata
    let ctx = FetchContext::new(&cli);
    let registry = ModuleRegistry::new();
    let total_start = std::time::Instant::now();
    let (outputs, timings) = registry.collect_all_timed(&ctx);
    let total_elapsed = total_start.elapsed();

    // JSON export mode skips ANSI styling and ASCII logo formatting entirely
    if cli.json {
        println!("{}", render_json(&outputs));
        if cli.timings {
            eprintln!("\n=== Module Execution Timings ===");
            for (mod_id, dur) in &timings {
                let micros = dur.as_micros();
                if micros < 1000 {
                    eprintln!("  {:<14} : {:>6} µs", mod_id.as_str(), micros);
                } else {
                    eprintln!(
                        "  {:<14} : {:>6.2} ms",
                        mod_id.as_str(),
                        dur.as_secs_f64() * 1000.0
                    );
                }
            }
            eprintln!("--------------------------------");
            eprintln!(
                "  {:<14} : {:>6.2} ms (parallel wall clock)",
                "Total Time",
                total_elapsed.as_secs_f64() * 1000.0
            );
        }
        return;
    }

    // Resolve distro ASCII art using explicit CLI override, distro ID, or ID_LIKE fallback
    let logo = if ctx.no_logo {
        None
    } else {
        match_logo(
            ctx.logo_override.as_deref(),
            &ctx.os_info.distro_id,
            &ctx.os_info.distro_like,
        )
    };

    // Format side-by-side or stacked layout depending on terminal column width
    let rendered = render_layout(logo, &outputs, ctx.term_width, ctx.enable_color);
    if !rendered.is_empty() {
        println!("{}", rendered);
    }

    if cli.timings {
        let cyan = if ctx.enable_color { "\x1b[1;36m" } else { "" };
        let reset = if ctx.enable_color { "\x1b[0m" } else { "" };
        println!("\n{}=== Module Execution Timings ==={}", cyan, reset);
        for (mod_id, dur) in &timings {
            let micros = dur.as_micros();
            if micros < 1000 {
                println!("  {:<14} : {:>6} µs", mod_id.as_str(), micros);
            } else {
                println!(
                    "  {:<14} : {:>6.2} ms",
                    mod_id.as_str(),
                    dur.as_secs_f64() * 1000.0
                );
            }
        }
        println!("{}--------------------------------{}", cyan, reset);
        println!(
            "  {:<14} : {:>6.2} ms (parallel wall clock)",
            "Total Time",
            total_elapsed.as_secs_f64() * 1000.0
        );
    }
}
