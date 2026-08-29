use crate::cli::Cli;
use crate::modules::os::{detect_os, OsInfo};
use crate::modules::ModuleId;
use std::io::IsTerminal;

pub struct FetchContext {
    pub term_width: u16,
    pub enable_color: bool,
    pub os_info: OsInfo,
    pub disk_target_path: String,
    pub active_modules: Vec<ModuleId>,
    pub logo_override: Option<String>,
    pub no_logo: bool,
}

impl FetchContext {
    pub fn new(cli: &Cli) -> Self {
        let term_width = get_terminal_width();
        let enable_color = should_enable_color(cli.no_color);
        let os_info = detect_os();
        let active_modules = resolve_active_modules(cli);

        Self {
            term_width,
            enable_color,
            os_info,
            disk_target_path: cli.disk_path.clone(),
            active_modules,
            logo_override: cli.logo.clone(),
            no_logo: cli.no_logo,
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

/// Determines whether colored output should be produced.
pub fn should_enable_color(no_color_flag: bool) -> bool {
    if no_color_flag {
        return false;
    }
    // Compliance with https://no-color.org: any non-empty or empty NO_COLOR disables ANSI styling
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    // Disable styling on dumb terminals to avoid emitting raw escape sequences to teletypes
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }
    // Force color flags take precedence over tty detection for CI runners and piped test logs
    if std::env::var_os("CLICOLOR_FORCE").is_some() || std::env::var_os("FORCE_COLOR").is_some() {
        return true;
    }
    std::io::stdout().is_terminal()
}

/// Resolves the active list of modules based on CLI flags.
pub fn resolve_active_modules(cli: &Cli) -> Vec<ModuleId> {
    let base_modules: Vec<ModuleId> = if let Some(ref mods) = cli.modules {
        mods.iter().filter_map(|m| ModuleId::from_str(m)).collect()
    } else {
        ModuleId::all().to_vec()
    };

    let filtered: Vec<ModuleId> = if let Some(ref disabled) = cli.disable {
        let disabled_set: Vec<ModuleId> = disabled
            .iter()
            .filter_map(|d| ModuleId::from_str(d))
            .collect();
        base_modules
            .into_iter()
            .filter(|m| !disabled_set.contains(m))
            .collect()
    } else {
        base_modules
    };

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

        let active = resolve_active_modules(&cli);
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

        let active = resolve_active_modules(&cli);
        assert_eq!(active, vec![ModuleId::Os, ModuleId::Memory]);
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

        let active = resolve_active_modules(&cli);
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

        let active = resolve_active_modules(&cli);
        assert!(active.is_empty());
    }

    #[test]
    fn test_should_enable_color_flags() {
        assert!(!should_enable_color(true));
    }
}
