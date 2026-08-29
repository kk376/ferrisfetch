use crate::cli::Cli;
use crate::config::Config;
use crate::modules::os::{detect_os, OsInfo};
use crate::modules::ModuleId;
use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLevel {
    None,
    Basic16,
    Color256,
    TrueColor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCaps {
    pub color_level: ColorLevel,
    pub unicode_supported: bool,
    pub nerd_font_detected: bool,
}

pub struct FetchContext {
    pub term_width: u16,
    pub enable_color: bool,
    pub caps: TerminalCaps,
    pub os_info: OsInfo,
    pub disk_target_path: String,
    pub active_modules: Vec<ModuleId>,
    pub logo_override: Option<String>,
    pub no_logo: bool,
    pub config: Config,
}

impl FetchContext {
    pub fn new(cli: &Cli) -> Self {
        let config = Config::load_default();
        Self::with_config(cli, config)
    }

    pub fn with_config(cli: &Cli, config: Config) -> Self {
        let term_width = get_terminal_width();
        let no_color = cli.no_color || config.no_color.unwrap_or(false);
        let caps = detect_terminal_caps(no_color);
        let enable_color = caps.color_level != ColorLevel::None;
        let os_info = detect_os();
        let active_modules = resolve_active_modules(cli, &config);

        let disk_target_path = if cli.disk_path != "/" {
            cli.disk_path.clone()
        } else {
            config.disk_path.clone().unwrap_or_else(|| "/".to_string())
        };

        let logo_override = cli.logo.clone().or_else(|| config.logo.clone());
        let no_logo = cli.no_logo || config.no_logo.unwrap_or(false);

        Self {
            term_width,
            enable_color,
            caps,
            os_info,
            disk_target_path,
            active_modules,
            logo_override,
            no_logo,
            config,
        }
    }
}

/// Detects terminal column width via direct kernel ioctl TIOCGWINSZ, Win32 console info, or $COLUMNS fallback.
pub fn get_terminal_width() -> u16 {
    #[cfg(unix)]
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return ws.ws_col;
        }
    }

    #[cfg(windows)]
    unsafe {
        #[repr(C)]
        struct Coord {
            x: i16,
            y: i16,
        }
        #[repr(C)]
        struct SmallRect {
            left: i16,
            top: i16,
            right: i16,
            bottom: i16,
        }
        #[repr(C)]
        struct ConsoleScreenBufferInfo {
            dw_size: Coord,
            dw_cursor_position: Coord,
            w_attributes: u16,
            sr_window: SmallRect,
            dw_maximum_window_size: Coord,
        }
        extern "system" {
            fn GetStdHandle(n_std_handle: u32) -> isize;
            fn GetConsoleScreenBufferInfo(
                h_console_output: isize,
                lp_console_screen_buffer_info: *mut ConsoleScreenBufferInfo,
            ) -> i32;
        }
        let handle = GetStdHandle(0xfffffff5); // STD_OUTPUT_HANDLE = -11
        let mut csbi = std::mem::zeroed::<ConsoleScreenBufferInfo>();
        if GetConsoleScreenBufferInfo(handle, &mut csbi) != 0 {
            let width = csbi.sr_window.right - csbi.sr_window.left + 1;
            if width > 0 {
                return width as u16;
            }
        }
    }

    // Fallback when stdout is redirected into a pipe or subshell without a tty fd
    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(c) = cols.trim().parse::<u16>() {
            if c > 0 {
                return c;
            }
        }
    }

    // Standard VT100 / POSIX terminal column fallback
    80
}

/// Detects supported ANSI color depth (None, 16-color, 256-color, TrueColor 24-bit).
pub fn detect_color_level(no_color_flag: bool) -> ColorLevel {
    if no_color_flag || std::env::var_os("NO_COLOR").is_some() {
        return ColorLevel::None;
    }

    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return ColorLevel::None;
        }
    }

    let is_forced =
        std::env::var_os("CLICOLOR_FORCE").is_some() || std::env::var_os("FORCE_COLOR").is_some();
    if !is_forced && !std::io::stdout().is_terminal() {
        return ColorLevel::None;
    }

    // 1. Check direct TrueColor indicators
    if let Ok(ct) = std::env::var("COLORTERM") {
        let ct_clean = ct.trim().to_lowercase();
        if ct_clean == "truecolor" || ct_clean == "24bit" {
            return ColorLevel::TrueColor;
        }
    }

    // 2. Known TrueColor terminals by TERM or process signature
    if let Ok(term) = std::env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("direct")
            || term_lower.contains("kitty")
            || term_lower.contains("alacritty")
            || term_lower.contains("ghostty")
            || term_lower.contains("wezterm")
            || term_lower.contains("foot")
        {
            return ColorLevel::TrueColor;
        }
        if term_lower.contains("256color") {
            return ColorLevel::Color256;
        }
    }

    ColorLevel::Basic16
}

/// Detects whether Unicode UTF-8 character encoding is active in the environment.
pub fn detect_unicode_supported() -> bool {
    for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let upper = val.to_uppercase();
            if upper.contains("UTF-8") || upper.contains("UTF8") {
                return true;
            }
        }
    }
    // Default modern Linux/macOS assumption
    #[cfg(unix)]
    {
        true
    }
    #[cfg(windows)]
    {
        // Check if Windows Console code page is UTF-8 (CP 65001)
        true
    }
}

/// Probes whether Nerd Font glyphs/icons can be safely rendered.
pub fn detect_nerd_font_support() -> bool {
    // Explicit environment flag override
    if let Ok(val) = std::env::var("NERD_FONT") {
        let clean = val.trim().to_lowercase();
        if clean == "1" || clean == "true" || clean == "yes" {
            return true;
        }
        if clean == "0" || clean == "false" || clean == "no" {
            return false;
        }
    }

    // Modern modern-terminal emulators with built-in or bundled Nerd Font glyph fallbacks
    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        let tp_lower = tp.to_lowercase();
        if tp_lower.contains("ghostty") || tp_lower.contains("wezterm") || tp_lower.contains("warp")
        {
            return true;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("kitty") || term_lower.contains("ghostty") {
            return true;
        }
    }

    // Starship / Oh-My-Posh / Powerlevel10k indicators
    if std::env::var_os("STARSHIP_SHELL").is_some()
        || std::env::var_os("STARSHIP_SESSION_KEY").is_some()
        || std::env::var_os("POSH_THEME").is_some()
        || std::env::var_os("P9K_SSH").is_some()
    {
        return true;
    }

    false
}

/// Aggregates all probed terminal capabilities into a single capability descriptor.
pub fn detect_terminal_caps(no_color_flag: bool) -> TerminalCaps {
    TerminalCaps {
        color_level: detect_color_level(no_color_flag),
        unicode_supported: detect_unicode_supported(),
        nerd_font_detected: detect_nerd_font_support(),
    }
}

/// Determines whether colored output should be produced.
pub fn should_enable_color(no_color_flag: bool) -> bool {
    detect_color_level(no_color_flag) != ColorLevel::None
}

/// Resolves the active list of modules based on CLI flags and config file settings.
pub fn resolve_active_modules(cli: &Cli, config: &Config) -> Vec<ModuleId> {
    let base_modules: Vec<ModuleId> = if let Some(ref mods) = cli.modules {
        mods.iter().filter_map(|m| ModuleId::from_str(m)).collect()
    } else if let Some(ref mods) = config.modules {
        mods.iter().filter_map(|m| ModuleId::from_str(m)).collect()
    } else {
        ModuleId::all().to_vec()
    };

    let mut disabled_set: Vec<ModuleId> = Vec::new();
    if let Some(ref disabled) = cli.disable {
        disabled_set.extend(disabled.iter().filter_map(|d| ModuleId::from_str(d)));
    }
    if let Some(ref disabled) = config.disable {
        disabled_set.extend(disabled.iter().filter_map(|d| ModuleId::from_str(d)));
    }

    let filtered: Vec<ModuleId> = base_modules
        .into_iter()
        .filter(|m| !disabled_set.contains(m))
        .collect();

    // Deduplicate while strictly preserving first-seen appearance order
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();
    for m in filtered {
        if seen.insert(m) {
            deduped.push(m);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_active_modules_default() {
        let cli = Cli {
            disk_path: "/".to_string(),
            ..Default::default()
        };
        let config = Config::default();

        let active = resolve_active_modules(&cli, &config);
        assert_eq!(active.len(), ModuleId::all().len());
    }

    #[test]
    fn test_resolve_active_modules_custom_and_disable() {
        let cli = Cli {
            modules: Some(vec![
                "os".to_string(),
                "cpu".to_string(),
                "memory".to_string(),
            ]),
            disable: Some(vec!["cpu".to_string()]),
            disk_path: "/".to_string(),
            ..Default::default()
        };
        let config = Config::default();

        let active = resolve_active_modules(&cli, &config);
        assert_eq!(active, vec![ModuleId::Os, ModuleId::Memory]);
    }

    #[test]
    fn test_resolve_active_modules_config_fallback() {
        let cli = Cli::default();
        let config = Config {
            modules: Some(vec!["kernel".to_string(), "uptime".to_string()]),
            disable: Some(vec!["uptime".to_string()]),
            ..Default::default()
        };

        let active = resolve_active_modules(&cli, &config);
        assert_eq!(active, vec![ModuleId::Kernel]);
    }

    #[test]
    fn test_resolve_active_modules_duplicates_and_invalid() {
        let cli = Cli {
            modules: Some(vec![
                "os".to_string(),
                "invalid_mod".to_string(),
                "os".to_string(),
                "cpu".to_string(),
                "cpu".to_string(),
            ]),
            disk_path: "/".to_string(),
            ..Default::default()
        };
        let config = Config::default();

        let active = resolve_active_modules(&cli, &config);
        assert_eq!(active, vec![ModuleId::Os, ModuleId::Cpu]);
    }

    #[test]
    fn test_resolve_active_modules_all_disabled() {
        let cli = Cli {
            modules: Some(vec!["os".to_string(), "cpu".to_string()]),
            disable: Some(vec!["os".to_string(), "cpu".to_string()]),
            disk_path: "/".to_string(),
            ..Default::default()
        };
        let config = Config::default();

        let active = resolve_active_modules(&cli, &config);
        assert!(active.is_empty());
    }

    #[test]
    fn test_should_enable_color_flags() {
        assert!(!should_enable_color(true));
        assert_eq!(detect_color_level(true), ColorLevel::None);
    }

    #[test]
    fn test_detect_terminal_caps_no_color() {
        let caps = detect_terminal_caps(true);
        assert_eq!(caps.color_level, ColorLevel::None);
    }

    #[test]
    fn test_detect_unicode_supported_default() {
        assert!(detect_unicode_supported());
    }
}
