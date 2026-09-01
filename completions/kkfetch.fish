# Fish shell completion script for kkfetch

# Disable file completions by default for kkfetch
complete -c kkfetch -f

# Modules definitions
set -l modules \
    'title\tUsername@Hostname title header' \
    'os\tOperating system and distribution name' \
    'host\tHardware product and model name' \
    'kernel\tLinux kernel release version' \
    'installed\tOS installation date and relative system age' \
    'uptime\tSystem running time since boot' \
    'packages\tInstalled package counts' \
    'pkgs\tInstalled package counts (alias)' \
    'shell\tCurrent shell name and version' \
    'display\tDisplay resolution and refresh rate' \
    'desktop\tDesktop Environment or Window Manager' \
    'de\tDesktop Environment (alias)' \
    'wm\tWindow Manager (alias)' \
    'terminal\tActive terminal emulator' \
    'term\tTerminal emulator (alias)' \
    'cpu\tProcessor model and core count' \
    'gpu\tDetected graphics hardware' \
    'memory\tSystem RAM usage statistics' \
    'mem\tSystem RAM usage (alias)' \
    'swap\tSwap space memory usage' \
    'disk\tTarget filesystem storage usage' \
    'battery\tBattery capacity and power status' \
    'localip\tLocal network IP address' \
    'theme\tGTK, Qt, or Desktop UI Theme' \
    'icons\tIcon theme name' \
    'colors\tTerminal 16-color ANSI palette' \
    'palette\tTerminal color palette (alias)'

# Logos definitions
set -l logos \
    'ferris\tFerris the Crab (Rust mascot)' \
    'rust\tFerris the Crab (alias)' \
    'debian\tDebian swirl logo' \
    'ubuntu\tUbuntu circle of friends logo' \
    'linuxmint\tLinux Mint logo' \
    'mint\tLinux Mint logo (alias)' \
    'fedora\tFedora infinity logo' \
    'arch\tArch Linux logo' \
    'archlinux\tArch Linux logo (alias)' \
    'rhel\tRed Hat Enterprise Linux logo' \
    'redhat\tRed Hat Enterprise Linux (alias)' \
    'centos\tCentOS logo (alias)' \
    'rocky\tRocky Linux logo' \
    'rockylinux\tRocky Linux logo (alias)' \
    'almalinux\tAlmaLinux logo' \
    'alma\tAlmaLinux logo (alias)' \
    'endeavouros\tEndeavourOS logo' \
    'endeavour\tEndeavourOS logo (alias)' \
    'manjaro\tManjaro Linux logo' \
    'opensuse\topenSUSE chameleon logo' \
    'suse\topenSUSE logo (alias)' \
    'alpine\tAlpine Linux mountain logo' \
    'gentoo\tGentoo Linux logo' \
    'void\tVoid Linux logo' \
    'pop\tPop!_OS logo' \
    'popos\tPop!_OS logo (alias)' \
    'nixos\tNixOS snowflake logo' \
    'nix\tNixOS logo (alias)' \
    'kali\tKali Linux dragon/shield logo' \
    'freebsd\tFreeBSD devil horns logo' \
    'slackware\tSlackware Linux logo' \
    'artix\tArtix Linux logo' \
    'zorin\tZorin OS logo' \
    'generic\tGeneric Linux Tux logo' \
    'tux\tTux penguin (alias)' \
    'linux\tGeneric Linux (alias)' \
    'none\tDisable ASCII logo output'

# Options and flags
complete -c kkfetch -s m -l modules -d 'Enable specific modules in order (comma-separated)' -r -a "$modules"
complete -c kkfetch -s d -l disable -d 'Disable specific modules (comma-separated)' -r -a "$modules"
complete -c kkfetch -s l -l logo -d 'Override the ASCII logo' -r -a "$logos"
complete -c kkfetch -l disk-path -d 'Target mount point or directory path for disk usage statistics' -r -F
complete -c kkfetch -l no-color -d 'Disable colored ANSI output'
complete -c kkfetch -l no-logo -d 'Do not display any ASCII logo'
complete -c kkfetch -l list-modules -d 'List all available information modules and exit'
complete -c kkfetch -l json -d 'Output system information in structured JSON format'
complete -c kkfetch -s h -l help -d 'Print help'
complete -c kkfetch -s V -l version -d 'Print version'
