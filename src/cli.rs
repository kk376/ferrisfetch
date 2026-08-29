use clap::Parser;

#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "ferrisfetch",
    about = "A fast, lightweight Linux system information fetch tool written in Rust",
    version
)]
pub struct Cli {
    /// Enable specific modules in order (comma-separated, e.g. "os,kernel,cpu,memory")
    #[arg(short = 'm', long = "modules", value_delimiter = ',')]
    pub modules: Option<Vec<String>>,

    /// Disable specific modules (comma-separated, e.g. "gpu,disk")
    #[arg(short = 'd', long = "disable", value_delimiter = ',')]
    pub disable: Option<Vec<String>>,

    /// Disable colored ANSI output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Override the ASCII logo (e.g. "arch", "debian", "ubuntu", "ferris", "fedora", "tux")
    #[arg(short = 'l', long = "logo")]
    pub logo: Option<String>,

    /// Do not display any ASCII logo
    #[arg(long = "no-logo")]
    pub no_logo: bool,

    /// List all available information modules and exit
    #[arg(long = "list-modules")]
    pub list_modules: bool,

    /// Target mount point or directory path for disk usage statistics (default: "/")
    #[arg(long = "disk-path", default_value = "/")]
    pub disk_path: String,

    /// Output system information in structured JSON format
    #[arg(long = "json")]
    pub json: bool,

    /// Show execution latency breakdown per module in microseconds
    #[arg(long = "timings")]
    pub timings: bool,
}
