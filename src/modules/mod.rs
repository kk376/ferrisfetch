use std::str::FromStr;

pub mod battery;
pub mod colors;
pub mod cpu;
pub mod desktop;
pub mod disk;
pub mod display;
pub mod gpu;
pub mod installed;
pub mod kernel;
pub mod localip;
pub mod memory;
pub mod os;
pub mod packages;
pub mod plugin;
pub mod shell;
pub mod terminal;
pub mod theme;
pub mod title;
pub mod uptime;
pub mod win_util;
pub mod wm;

use crate::context::FetchContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleId {
    Title,
    Os,
    Host,
    Kernel,
    Installed,
    Uptime,
    Packages,
    Shell,
    Display,
    Desktop,
    Wm,
    WmTheme,
    Terminal,
    TerminalFont,
    Cpu,
    Gpu,
    Memory,
    Swap,
    Disk,
    Battery,
    LocalIp,
    Theme,
    Icons,
    Font,
    Plugin,
    Colors,
}

impl ModuleId {
    pub fn all() -> &'static [ModuleId] {
        &[
            ModuleId::Title,
            ModuleId::Os,
            ModuleId::Host,
            ModuleId::Kernel,
            ModuleId::Installed,
            ModuleId::Uptime,
            ModuleId::Packages,
            ModuleId::Shell,
            ModuleId::Display,
            ModuleId::Desktop,
            ModuleId::Wm,
            ModuleId::WmTheme,
            ModuleId::Terminal,
            ModuleId::TerminalFont,
            ModuleId::Cpu,
            ModuleId::Gpu,
            ModuleId::Memory,
            ModuleId::Swap,
            ModuleId::Disk,
            ModuleId::Battery,
            ModuleId::LocalIp,
            ModuleId::Theme,
            ModuleId::Icons,
            ModuleId::Font,
            ModuleId::Plugin,
            ModuleId::Colors,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleId::Title => "title",
            ModuleId::Os => "os",
            ModuleId::Host => "host",
            ModuleId::Kernel => "kernel",
            ModuleId::Installed => "installed",
            ModuleId::Uptime => "uptime",
            ModuleId::Packages => "packages",
            ModuleId::Shell => "shell",
            ModuleId::Display => "display",
            ModuleId::Desktop => "desktop",
            ModuleId::Wm => "wm",
            ModuleId::WmTheme => "wmtheme",
            ModuleId::Terminal => "terminal",
            ModuleId::TerminalFont => "terminalfont",
            ModuleId::Cpu => "cpu",
            ModuleId::Gpu => "gpu",
            ModuleId::Memory => "memory",
            ModuleId::Swap => "swap",
            ModuleId::Disk => "disk",
            ModuleId::Battery => "battery",
            ModuleId::LocalIp => "localip",
            ModuleId::Theme => "theme",
            ModuleId::Icons => "icons",
            ModuleId::Font => "font",
            ModuleId::Plugin => "plugin",
            ModuleId::Colors => "colors",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<ModuleId> {
        s.parse::<ModuleId>().ok()
    }
}

impl FromStr for ModuleId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "title" => Ok(ModuleId::Title),
            "os" => Ok(ModuleId::Os),
            "host" => Ok(ModuleId::Host),
            "kernel" => Ok(ModuleId::Kernel),
            "installed" | "install" | "installdate" | "osinstall" => Ok(ModuleId::Installed),
            "uptime" => Ok(ModuleId::Uptime),
            "packages" | "pkgs" => Ok(ModuleId::Packages),
            "shell" => Ok(ModuleId::Shell),
            "display" | "resolution" | "screen" => Ok(ModuleId::Display),
            "desktop" | "de" => Ok(ModuleId::Desktop),
            "wm" | "windowmanager" => Ok(ModuleId::Wm),
            "wmtheme" | "wm_theme" => Ok(ModuleId::WmTheme),
            "terminal" | "term" => Ok(ModuleId::Terminal),
            "terminalfont" | "terminal_font" | "termfont" | "term_font" => Ok(ModuleId::TerminalFont),
            "cpu" => Ok(ModuleId::Cpu),
            "gpu" => Ok(ModuleId::Gpu),
            "memory" | "mem" => Ok(ModuleId::Memory),
            "swap" => Ok(ModuleId::Swap),
            "disk" => Ok(ModuleId::Disk),
            "battery" | "bat" => Ok(ModuleId::Battery),
            "localip" | "local_ip" | "ip" => Ok(ModuleId::LocalIp),
            "theme" | "gtk" | "gtktheme" => Ok(ModuleId::Theme),
            "icons" | "icontheme" => Ok(ModuleId::Icons),
            "font" | "fonts" | "gtkfont" => Ok(ModuleId::Font),
            "plugin" | "plugins" | "custom" => Ok(ModuleId::Plugin),
            "colors" | "palette" => Ok(ModuleId::Colors),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleOutput {
    pub id: ModuleId,
    pub label: String,
    pub value: String,
    pub custom_rendered: Option<String>,
}

/// System metric collector trait. Supports both single-output and multi-output collectors (e.g. multi-GPU, multi-disk).
pub trait Collector: Send + Sync {
    fn id(&self) -> ModuleId;
    fn collect(&self, ctx: &FetchContext) -> Option<ModuleOutput> {
        self.collect_multiple(ctx).into_iter().next()
    }
    fn collect_multiple(&self, ctx: &FetchContext) -> Vec<ModuleOutput> {
        self.collect(ctx).into_iter().collect()
    }
}

pub struct ModuleRegistry {
    collectors: Vec<Box<dyn Collector>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let collectors: Vec<Box<dyn Collector>> = vec![
            Box::new(title::TitleCollector),
            Box::new(os::OsCollector),
            Box::new(os::HostCollector),
            Box::new(kernel::KernelCollector),
            Box::new(installed::InstalledCollector),
            Box::new(uptime::UptimeCollector),
            Box::new(packages::PackagesCollector),
            Box::new(shell::ShellCollector),
            Box::new(display::DisplayCollector),
            Box::new(desktop::DesktopCollector),
            Box::new(wm::WmCollector),
            Box::new(wm::WmThemeCollector),
            Box::new(terminal::TerminalCollector),
            Box::new(terminal::TerminalFontCollector),
            Box::new(cpu::CpuCollector),
            Box::new(gpu::GpuCollector),
            Box::new(memory::MemoryCollector),
            Box::new(memory::SwapCollector),
            Box::new(disk::DiskCollector),
            Box::new(battery::BatteryCollector),
            Box::new(localip::LocalIpCollector),
            Box::new(theme::ThemeCollector),
            Box::new(theme::IconsCollector),
            Box::new(theme::FontCollector),
            Box::new(plugin::PluginCollector),
            Box::new(colors::ColorsCollector),
        ];

        Self { collectors }
    }

    /// Collects metrics from active modules concurrently using std::thread::scope while preserving deterministic ordering.
    pub fn collect_all(&self, ctx: &FetchContext) -> Vec<ModuleOutput> {
        self.collect_all_timed(ctx).0
    }

    /// Collects metrics from active modules concurrently while recording microsecond execution duration per module.
    pub fn collect_all_timed(
        &self,
        ctx: &FetchContext,
    ) -> (Vec<ModuleOutput>, Vec<(ModuleId, std::time::Duration)>) {
        let active = &ctx.active_modules;
        if active.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let results: Vec<(Vec<ModuleOutput>, ModuleId, std::time::Duration)> =
            std::thread::scope(|s| {
                let mut handles = Vec::with_capacity(active.len());
                for &module_id in active {
                    if let Some(collector) = self.collectors.iter().find(|c| c.id() == module_id) {
                        handles.push(s.spawn(move || {
                            let start = std::time::Instant::now();
                            let outputs = collector.collect_multiple(ctx);
                            let elapsed = start.elapsed();
                            (outputs, module_id, elapsed)
                        }));
                    }
                }
                handles
                    .into_iter()
                    .map(|h| {
                        h.join()
                            .unwrap_or_else(|_| (Vec::new(), ModuleId::Title, std::time::Duration::ZERO))
                    })
                    .collect()
            });

        let mut all_outputs = Vec::new();
        let mut timings = Vec::with_capacity(results.len());

        for (outs, mod_id, dur) in results {
            all_outputs.extend(outs);
            timings.push((mod_id, dur));
        }

        (all_outputs, timings)
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_id_from_str() {
        assert_eq!(ModuleId::from_str("os"), Some(ModuleId::Os));
        assert_eq!(ModuleId::from_str("installed"), Some(ModuleId::Installed));
        assert_eq!(ModuleId::from_str("mem"), Some(ModuleId::Memory));
        assert_eq!(ModuleId::from_str("pkgs"), Some(ModuleId::Packages));
        assert_eq!(ModuleId::from_str("theme"), Some(ModuleId::Theme));
        assert_eq!(ModuleId::from_str("icons"), Some(ModuleId::Icons));
        assert_eq!(ModuleId::from_str("font"), Some(ModuleId::Font));
        assert_eq!(ModuleId::from_str("wmtheme"), Some(ModuleId::WmTheme));
        assert_eq!(ModuleId::from_str("terminalfont"), Some(ModuleId::TerminalFont));
        assert_eq!(ModuleId::from_str("plugin"), Some(ModuleId::Plugin));
        assert_eq!(ModuleId::from_str("palette"), Some(ModuleId::Colors));
        assert_eq!(ModuleId::from_str("invalid_mod"), None);
    }

    #[test]
    fn test_module_id_all_count() {
        assert_eq!(ModuleId::all().len(), 26);
    }
}
