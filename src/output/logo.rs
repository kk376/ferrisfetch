//! ASCII Art Logos and ANSI Color Layout Engine.
//!
//! Distribution ASCII art boundary outlines are based on classic designs from
//! Neofetch (MIT License, Copyright (c) 2016-2022 Dylan Araps), enhanced in
//! KKFetch with high-contrast white structural framing (`{w}`) and
//! distribution brand signature color emblems (`{c}`).

use crate::output::color::RESET;

pub const WHITE_COLOR: &str = "\x1b[38;5;231m";

#[derive(Debug, Clone)]
pub struct Logo {
    pub name: &'static str,
    pub raw_lines: &'static [&'static str],
    pub distro_color: &'static str,
    pub outer_color: &'static str,
}

impl Logo {
    /// Returns the logo lines formatted with ANSI colors if enabled.
    /// The outer layer is styled with `outer_color` (white), and the
    /// highlighted inner emblem is styled with `distro_color`.
    pub fn render_lines(&self, enable_color: bool) -> Vec<String> {
        self.raw_lines
            .iter()
            .map(|line| {
                if enable_color {
                    let mut rendered = line
                        .replace("{c}", self.distro_color)
                        .replace("{w}", self.outer_color)
                        .replace("{0}", RESET);
                    if !line.contains("{c}") && !line.contains("{w}") && !line.is_empty() {
                        rendered = format!("{}{}{}", self.distro_color, line, RESET);
                    } else if !line.is_empty() {
                        rendered.push_str(RESET);
                    }
                    rendered
                } else {
                    line.replace("{c}", "")
                        .replace("{w}", "")
                        .replace("{0}", "")
                }
            })
            .collect()
    }
}

pub const ALL_LOGOS: &[Logo] = &[
    Logo {
        name: "ferris",
        raw_lines: &[
            "        {w}/^^^\\     /^^^\\",
            "       {w}(  O  )   (  O  )",
            "      {c}.-'---'-----'---'-.",
            "     {c}/   _~^~^~^~^~_     \\",
            "    {c}|   /  {w}o     o{c}  \\     |",
            "    {c}|   |     {w}-{c}     |     |",
            "    {c}|   \\  {w}'---'{c}  /     |",
            "     {c}\\   '-------'     /",
            "      {c}'-._'-------'_.-'",
            "          {w}/       \\",
        ],
        distro_color: "\x1b[38;5;208m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "debian",
        raw_lines: &[
            "{c}       _,met$$$$$gg.",
            "{c}    ,g$$$$$$$$$$$$$$$P.",
            "{c}  ,g$$P\"        \"\"\"Y$$.\".",
            "{c} ,$$P'              `$$$.",
            "{c}',$$P       {w},ggs.{c}     `$$b:",
            "{c}`d$$'     {w},$P\"'   .{c}    $$$",
            "{c} $$P      {w}d$'     ,{c}    $$P",
            "{c} $$:      {w}$$.   -{c}    ,d$$'",
            "{c} $$;      {w}Y$b._   _,d$P'{c}",
            "{c} Y$$.    {w}`.`\"Y$$$$P\"'{c}",
            "{c} `$$b      {w}\"-.__{c}",
            "{c}  `Y$$",
            "{c}   `Y$$.",
            "{c}     `$$b.",
            "{c}       `Y$$b.",
            "{c}          `\"Y$b._",
            "{c}              `\"\"\"",
        ],
        distro_color: "\x1b[38;5;196m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "ubuntu",
        raw_lines: &[
            "{w}            .-/+oossssoo+\\-.",
            "{w}        ´:+ssssssssssssssssss+:`",
            "{w}      -+ssssssssssssssssssyyssss+-",
            "{w}    .ossssssssssssssssss{c}dMMMNy{w}sssso.",
            "{w}   /sssssssssss{c}hdmmNNmmyNMMMMh{w}ssssss\\",
            "{w}  +sssssssss{c}hm{w}yd{c}MMMMMMMNddddy{w}ssssssss+",
            "{w} /ssssssss{c}hNMMM{w}yh{c}hyyyyhmNMMMNh{w}ssssssss\\",
            "{w}.ssssssss{c}dMMMNh{w}ssssssssss{c}hNMMMd{w}ssssssss.",
            "{w}+ssss{c}hhhyNMMNy{w}ssssssssssss{c}yNMMMy{w}sssssss+",
            "{w}oss{c}yNMMMNyMMh{w}ssssssssssssss{c}hmmmh{w}ssssssso",
            "{w}oss{c}yNMMMNyMMh{w}ssssssssssssss{c}hmmmh{w}ssssssso",
            "{w}+ssss{c}hhhyNMMNy{w}ssssssssssss{c}yNMMMy{w}sssssss+",
            "{w}.ssssssss{c}dMMMNh{w}ssssssssss{c}hNMMMd{w}ssssssss.",
            "{w} \\ssssssss{c}hNMMM{w}yh{c}hyyyyhdNMMMNh{w}ssssssss/",
            "{w}  +sssssssss{c}dm{w}yd{c}MMMMMMMMddddy{w}ssssssss+",
            "{w}   \\sssssssssss{c}hdmNNNNmyNMMMMh{w}ssssss/",
            "{w}    .ossssssssssssssssss{c}dMMMNy{w}sssso.",
            "{w}      -+sssssssssssssssss{c}yyy{w}ssss+-",
            "{w}        `:+ssssssssssssssssss+:`",
            "{w}            .-\\+oossssoo+/-.",
        ],
        distro_color: "\x1b[38;5;208m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "linuxmint",
        raw_lines: &[
            "{w}             ...-:::::-...",
            "{w}          .-MMMMMMMMMMMMMMM-.",
            "{w}      .-MMMM{c}`..-:::::::-..`{w}MMMM-.",
            "{w}    .:MMMM{c}.:MMMMMMMMMMMMMMM:.{w}MMMM:.",
            "{w}   -MMM{c}-M---MMMMMMMMMMMMMMMMMMM.{w}MMM-",
            "{w} `:MMM{c}:MM`  :MMMM:....::-...-MMMM:{w}MMM:`",
            "{w} :MMM{c}:MMM`  :MM:`  ``    ``  `:MMM:{w}MMM:",
            "{w}.MMM{c}.MMMM`  :MM.  -MM.  .MM-  `MMMM.{w}MMM.",
            "{w}:MMM{c}:MMMM`  :MM.  -MM-  .MM:  `MMMM-{w}MMM:",
            "{w}:MMM{c}:MMMM`  :MM.  -MM-  .MM:  `MMMM:{w}MMM:",
            "{w}:MMM{c}:MMMM`  :MM.  -MM-  .MM:  `MMMM-{w}MMM:",
            "{w}.MMM{c}.MMMM`  :MM:--:MM:--:MM:  `MMMM.{w}MMM.",
            "{w} :MMM{c}:MMM-  `-MMMMMMMMMMMM-`  -MMM-{w}MMM:",
            "{w}  :MMM{c}:MMM:`                `:MMM:{w}MMM:",
            "{w}   .MMM{c}.MMMM:--------------:MMMM.{w}MMM.",
            "{w}     '-MMMM{c}.-MMMMMMMMMMMMMMM-.{w}MMMM-'",
            "{w}       '.-MMMM{c}``--:::::--``{w}MMMM-.'",
            "{w}            '-MMMMMMMMMMMMM-'",
            "{w}               ``-:::::-``",
        ],
        distro_color: "\x1b[38;5;46m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "fedora",
        raw_lines: &[
            "{w}             .',;::::;,'.",
            "{w}         .';:cccccccccccc:;,.",
            "{w}      .;cccccccccccccccccccccc;.",
            "{w}    .:cccccccccccccccccccccccccc:.",
            "{w}  .;ccccccccccccc;{c}.:dddl:.{w};ccccccc;.",
            "{w} .:ccccccccccccc;{c}OWMKOOXMWd{w};ccccccc:.",
            "{w}.:ccccccccccccc;{c}KMMc{w};cc;{c}xMMc{w};ccccccc:.",
            "{w},cccccccccccccc;{c}MMM.{w};cc;{c};WW:{w};cccccccc,",
            "{w}:cccccccccccccc;{c}MMM.{w};cccccccccccccccc:",
            "{w}:ccccccc;{c}oxOOOo{w};{c}MMM0OOk.{w};cccccccccccc:",
            "{w}cccccc;{c}0MMKxdd:{w};{c}MMMkddc.{w};cccccccccccc;",
            "{w}ccccc;{c}XM0'{w};cccc;{c}MMM.{w};cccccccccccccccc'",
            "{w}ccccc;{c}MMo{w};ccccc;{c}MMW.{w};ccccccccccccccc;",
            "{w}ccccc;{c}0MNc.{w}ccc{c}.xMMd{w};ccccccccccccccc;",
            "{w}cccccc;{c}dNMWXXXWM0:{w};cccccccccccccc:,",
            "{w}cccccccc;{c}.:odl:.{w};cccccccccccccc:,.",
            "{w}:cccccccccccccccccccccccccccc:'.",
            "{w}.:cccccccccccccccccccccc:;,..",
            "{w}  '::cccccccccccccc::;,.",
        ],
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "arch",
        raw_lines: &[
            "{w}                   -`",
            "{w}                  .o+`",
            "{w}                 `ooo/",
            "{w}                `+oooo:",
            "{w}               `+oooooo:",
            "{w}               -+oooooo+:",
            "{w}             `/:-:++oooo+:",
            "{w}            `/++++/+++++++:",
            "{w}           `/++++++++++++++:",
            "{w}          `/+++o{c}oooooooo{w}oooo/`",
            "{w}         ./{c}ooosssso++osssssso{w}+`",
            "{w}        .{c}oossssso-````/ossssss{w}+`",
            "{w}       -{c}osssssso.      :ssssssso{w}.",
            "{w}      :{c}osssssss/        {w}osssso+++.",
            "{w}     /{c}ossssssss/        {w}+ssssooo/-",
            "{w}   `/{c}ossssso+/:-        {w}-:/+osssso+-",
            "{w}  `+sso+:-`                 `.-/+oso:",
            "{w} `++:.                           `-/+/",
            "{w} .`                                 `/",
        ],
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "rhel",
        raw_lines: &[
            "{w}             `.-..........`",
            "{w}            `////////::.`-/.",
            "{w}            -: ....-////////.",
            "{w}            //:-::///////////`",
            "{w}     `--::: `-://////////////:",
            "{w}     //////-    ``.-:///////// .`",
            "{w}     `://////:-.`    :///////::///:`",
            "{w}       .-/////////:---/////////////:",
            "{w}          .-://////////////////////.",
            "{c}         yMN+`.-{w}::///////////////-`",
            "{c}      .-`:NMMNMs`  {w}`..-------..`",
            "{c}       MN+/mMMMMMhoooyysshsss",
            "{c}MMM    MMMMMMMMMMMMMMyyddMMM+",
            "{c} MMMM   MMMMMMMMMMMMMNdyNMMh`     {w}hyhMMM",
            "{c}  MMMMMMMMMMMMMMMMyoNNNMMM+.   {w}MMMMMMMM",
            "{c}   MMNMMMNNMMMMMNM+ mhsMNyyyyMNMMMMsMM",
        ],
        distro_color: "\x1b[38;5;196m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "rocky",
        raw_lines: &[
            "{w}    `-/+++++++++/-.`",
            "{w} `-+++++++++++++++++-`",
            "{w}.+++++++++++++++++++++.",
            "{w}-+++++++++++++++++++++++.",
            "{c}+++++++++++++++{w}/-/{c}+++++++",
            "{c}+++++++++++++/.   {w}./+++++",
            "{c}+++++++++++:.       {w}./+++",
            "{c}+++++++++:`   `:/:`   {w}.:/",
            "{c}-++++++:`   .:+++++:`",
            "{c} .+++-`   ./+++++++++:`",
            "{c}  `-`   ./+++++++++++-",
            "{c}       -+++++++++:-.`",
        ],
        distro_color: "\x1b[38;5;35m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "almalinux",
        raw_lines: &[
            "{w}         'c:.",
            "{w}        lkkkx, ..       {c}..   ,cc,",
            "{w}        okkkk:ckkx'  {c}.lxkkx.okkkkd",
            "{w}        .:llcokkx'  {c}:kkkxkko:xkkd,",
            "{w}      .xkkkkdood:  {c};kx,  .lkxlll;",
            "{w}       xkkx.       {c}xk'     xkkkkk:",
            "{w}       'xkx.       {c}xd      .....,.",
            "{c}      .. {w}:xkl'     {c}:c      ..''..",
            "{c}    .dkx'  {w}.:ldl:'. {c}'  ':lollldkkxo;",
            "{c}  .''lkkko'                     ckkkx.",
            "{c}'xkkkd:kkd.       ..  {w};'        {c}:kkxo.",
            "{c},xkkkd;kk'      ,d;    {w}ld.   {c}':dkd::cc,",
            "{c} .,,.;xkko'.';lxo.      {w}dx,  {c}:kkk'xkkkkc",
            "{c}     'dkkkkkxo:.        {w};kx  {c}.kkk:;xkkd.",
            "{c}       .....   {w}.;dk:.   lkk.  {c}:;,",
            "{w}             :kkkkkkkdoxkkx",
            "{w}              ,c,,;;;:xkkd.",
            "{w}                ;kkkkl...",
            "{w}                ;kkkkl",
            "{w}                 ,od;",
        ],
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "endeavouros",
        raw_lines: &[
            "{w}                     ./{c}o{w}.",
            "{w}                   ./{c}sssso{w}-",
            "{w}                 `:{c}osssssss+{w}-",
            "{w}               `:+{c}sssssssssso{w}/.",
            "{w}             `-/o{c}ssssssssssssso{w}/.",
            "{w}           `-/+{c}sssssssssssssssso{w}+:`",
            "{w}         `-:/+{c}sssssssssssssssssso{w}+/.",
            "{w}       `.://o{c}sssssssssssssssssssso{w}++-",
            "{w}      .://+{c}ssssssssssssssssssssssso{w}++:",
            "{w}    .:///o{c}ssssssssssssssssssssssssso{w}++:",
            "{w}  `:////{c}ssssssssssssssssssssssssssso{w}+++.",
            "{w}`-////+{c}ssssssssssssssssssssssssssso{w}++++-",
            "{w} `..-+{c}oosssssssssssssssssssssssso{w}+++++/`",
            "{w}   ./++++++++++++++++++++++++++++++/:.",
            "{w}  `:::::::::::::::::::::::::------``",
        ],
        distro_color: "\x1b[38;5;127m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "manjaro",
        raw_lines: &[
            "{c}██████████████████  {w}████████",
            "{c}██████████████████  {w}████████",
            "{c}██████████████████  {w}████████",
            "{c}██████████████████  {w}████████",
            "{c}████████            {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
            "{c}████████  {w}████████  {w}████████",
        ],
        distro_color: "\x1b[38;5;34m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "generic",
        raw_lines: &[
            "{w}        #####",
            "{w}       #######",
            "{w}       ##{c}O{w}#{c}O{w}##",
            "{w}       #{w}#####{w}#",
            "{w}     ##{c}##{w}###{c}##{w}##",
            "{w}    #{c}#######{w}####{c}#",
            "{w}   #{c}#############{w}#",
            "{w}  #{c}###############{w}#",
            "{w} #{c}#################{w}#",
            "{w} #{c}#################{w}#",
            "{w} #{c}#################{w}#",
            "{w} #{c}#################{w}#",
            "{w}  #{c}###############{w}#",
            "{w}   {c}#{w}#{c}###########{w}#{c}#",
            "{c}    ####{w}#####{c}####",
            "{c}    ###       ###",
        ],
        distro_color: "\x1b[38;5;220m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "opensuse",
        raw_lines: &[
            "{w}           .;ldkO0000Okdl;.",
            "{w}         .;d00xl:^''''''^:ok00d;.",
            "{w}       .d00l'                'o00d.",
            "{w}     .d0Kd'{c}  Okxol:;,.          {w}:O0d.",
            "{w}    .d{c}0Kx.  oOO00kkkxl:.         {w}:EKx.",
            "{w}   .o{c}0Kx.  l00000000000kx:.       {w}.K0o.",
            "{w}   :{c}0Kx.  l00000000000000000kx;     {w}K0:",
            "{w}  .d{c}0Kx.  l000000000000000000000d.  {w}.EKx.",
            "{w}  .d{c}0Kx.  o0000000000000000000000o  {w}.EKx.",
            "{w}   :{c}0Kx.  l000000000000000000000o   {w}K0:",
            "{w}   .o{c}0Kx.  l000000000000000000kx;  {w}.K0o.",
            "{w}    .d{c}0Kx.  oOO00kkkxl:.         {w}:EKx.",
            "{w}     .d0Kd'{c}  Okxol:;,.          {w}:O0d.",
            "{w}       .d00l'                'o00d.",
            "{w}         .;d00xl:^''''''^:ok00d;.",
            "{w}           .;ldkO0000Okdl;.",
        ],
        distro_color: "\x1b[38;5;71m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "alpine",
        raw_lines: &[
            "{w}       .hddddddddddddddddddddddh.",
            "{w}      :dddddddddddddddddddddddddd:",
            "{w}     /dddddddddddddddddddddddddddd/",
            "{w}    +dddddddddddddddddddddddddddddd+",
            "{w}  `sdddddddddddddddddddddddddddddddds`",
            "{w} `yddddddddddddddddddddddddddddddddddy`",
            "{w}.hddddddddddddddddddddddddddddddddddddh.",
            "{w}+dddddddddddddddddddddddddddddddddddddd+",
            "{w} `yddddddddddddddddddddddddddddddddddy`",
            "{w}  `sdddddddddddddddddddddddddddddddds`",
            "{w}    +dddddddddddddddddddddddddddddd+",
            "{w}     /dddddddddddddddddddddddddddd/",
            "{w}      :dddddddddddddddddddddddddd:",
            "{w}       .hddddddddddddddddddddddh.",
        ],
        distro_color: "\x1b[38;5;32m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "gentoo",
        raw_lines: &[
            "{w}         -/oyddmdhs+:.",
            "{w}       -o{c}dNMMMMMMMMNNmhy+{w}-`",
            "{w}     -y{c}NMMMMMMMMMMMNNNmmdhy{w}+-",
            "{w}   `o{c}mMMMMMMMMMMMMNmdmmmmddhhy{w}/`",
            "{w}   om{c}MMMMMMMMMMMN{w}hhyyyo{c}hmdddhhhdN{w}`",
            "{w}  .yd{c}MMMMMMMMMMMM{w}hhhhhyo{c}yhhdddddhny{w}",
            "{w}  `sh{c}MMMMMMMMMMMM{w}hhhhhhhy{c}yyyyyhddddy{w}",
            "{w}   `d{c}NMMMMMMMMMMM{w}hhhhhhhy{c}yyyyyyddddh{w}`",
            "{w}    .d{c}NMMMMMMMMMM{w}yhhhhhhy{c}yyyyyhddddh{w}-",
            "{w}      -d{c}NMMMMMMMM{w}hhhhhyo{c}yyyyydddddy{w}+",
            "{w}       `+h{c}NMMMMMM{w}hyso/{c}yyyyydddddy{w}+",
            "{w}          `:+syd{w}hyyyyyhddddds{w}+",
            "{w}              `-:+syhhhysoo+.",
        ],
        distro_color: "\x1b[38;5;141m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "void",
        raw_lines: &[
            "{w}                __.;=====;.__",
            "{w}            _.=+==++=++=+=+===;.",
            "{w}             -=+++=+===+=+=+++++=_",
            "{w}        .     -=:``     `--==+=++==.",
            "{w}       :@=._                `-====+=",
            "{w}      ;@@@@@@={c}         .        {w}=*+",
            "{w}    .========{c}         :@@@=._     {w}=",
            "{w}   =++====++={c}        .@@@@@@@@=    {w}:",
            "{w}  :+++=++++=={c}        +@@@@@@@@@.   {w}.",
            "{w}  *+++=++++++{c}       .@@@@@@@@@@    {w}:",
            "{w}  :+++=+++++={c}       .@@@@@@@@@+    {w}.",
            "{w}   =++====++={c}        @@@@@@@@@     {w}:",
            "{w}    .========{c}        :@@@@@@@=    {w}=",
            "{w}      ;@@@@@@={c}         `*@@*'     {w}=*+",
            "{w}       :@=._                `-====+=",
            "{w}        .     -=:``     `--==+=++==.",
            "{w}             -=+++=+===+=+=+++++=_",
            "{w}            _.=+==++=++=+=+===;.",
            "{w}                __.;=====;.__",
        ],
        distro_color: "\x1b[38;5;35m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "pop",
        raw_lines: &[
            "{w}             `.-:::-.`",
            "{w}           -+ydmNNNNNNNmdy+-",
            "{w}        .+dNmdhs+//////+shdmdo.",
            "{w}      .smmy+-`             ./sdy:",
            "{w}     +mds-`                   -sdy`",
            "{w}    ymd-   {c}:++o-      -o++:    {w}-dmy",
            "{w}   hmh`    {c}:ydN+      +Ndy:     {w}`hmh",
            "{w}  -mm+     {c}:ydN+      +Ndy:      {w}+mm-",
            "{w}  +mm.     {c}:ydN+      +Ndy:      {w}.mm+",
            "{w}  -mm+     {c}:ydN+      +Ndy:      {w}+mm-",
            "{w}   hmh`    {c}:ydN+      +Ndy:     {w}`hmh",
            "{w}    ymd-   {c}:ydN+      +Ndy:    {w}-dmy",
            "{w}     +mds- {c}:ydN+      +Ndy:  {w}-sdy`",
            "{w}      .smmy{c}:ydN+      +Ndy:{w}./sdy:",
            "{w}        .+dN{c}shs+//////+shd{w}mdo.",
            "{w}           -+ydmNNNNNNNmdy+-",
            "{w}             `.-:::-.`",
        ],
        distro_color: "\x1b[38;5;37m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "nixos",
        raw_lines: &[
            "{w}          ▗▄▄▄       {c}▗▄▄▄▄    ▄▄▄▖",
            "{w}          ▜███▙       {c}▜███▙  ▟███▛",
            "{w}           ▜███▙       {c}▜███▙▟███▛",
            "{w}            ▜███▙       {c}▜██████▛",
            "{w}     ▟█▙     ▜███▙       {c}▜████▛",
            "{w}    ▟███▙     ▜███▙     {c}▟██████▙",
            "{w}   ▟█████▙     ▜███▙   {c}▟███▛▜███▙",
            "{w}  ▟███▛▜███▙    ▜███▙ {c}▟███▛  ▜███▙",
            "{w} ▟███▛  ▜███▙    ▜███{c}▟███▛    ▜███▙",
            "{w}▝▀▀▀▀    ▀▀▀▀▘    ▀▀▀▀▀▀▀      ▀▀▀▀▘",
            "{c}     ▟█████████████████▙ {w}▜████▛     {c}▟▙",
            "{c}    ▟███████████████████▙ {w}▜███▙    {c}▟██▙",
            "{c}           ▄▄▄▄▖           ▜███▙  {w}▟███▛",
            "{c}          ▟███▛             ▜██▛ {w}▟███▛",
            "{c}         ▟███▛               ▜▛ {w}▟███▛",
            "{c}▟███████████▛                  {w}▟██████████▙",
            "{c}▜██████████▛                  {w}▟███████████▛",
            "{c}      ▟███▛ {w}▟▙               {c}▟███▛",
            "{c}     ▟███▛ {w}▟██▙             {c}▟███▛",
            "{c}    ▟███▛  {w}▜███▙           {c}▝▀▀▀▀",
            "{c}    ▜██▛    {w}▜███▙ {c}▜██████████████████▛",
            "{c}     ▜▛     {w}▟████▙ {c}▜████████████████▛",
            "{w}           ▟██████▙       {c}▜███▙",
            "{w}          ▟███▛▜███▙       {c}▜███▙",
            "{w}         ▟███▛  ▜███▙       {c}▜███▙",
            "{w}         ▝▀▀▀    ▀▀▀▀▘       {c}▀▀▀▘",
        ],
        distro_color: "\x1b[38;5;75m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "kali",
        raw_lines: &[
            "{w}..............",
            "{w}            ..,;:ccc,.",
            "{w}          ......''';lxO.",
            "{w}.....''''..........,:ld;",
            "{w}           .';;;:::;,,.x,",
            "{w}      ..'''.            {c}0Xxoc:,.  ...{w}",
            "{w}  ....                {c},ONkc;,;cokOdc',.{w}",
            "{w} .                   {c}OMo           ':ddo.{w}",
            "{w}                    {c}dMc               :OO;{w}",
            "{w}                    {c}0M.                 .:o.{w}",
            "{w}                    {c};Wd{w}",
            "{w}                     {c};XO,{w}",
            "{w}                       {c},d0Odlc;,..{w}",
            "{w}                           {c}..',;:cdOOd::,.{w}",
            "{w}                                    {c}.:d;.':;.{w}",
            "{w}                                       {c}'d,  .'{w}",
            "{w}                                         {c};l   ..{w}",
            "{w}                                          {c}.o{w}",
            "{w}                                            {c}c{w}",
            "{w}                                            {c}.'{w}",
            "{w}                                             {c}.{w}",
        ],
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "freebsd",
        raw_lines: &[
            "{w}   ```                        {c}`",
            "{w}  ` `.....---...{c}....--.```   -/{w}",
            "{w}  +o   .--`         {c}/y:`      +.{w}",
            "{w}   yo`:.            {c}:o      `+-{w}",
            "{w}    y/               {c}-/`   -o/{w}",
            "{w}   .-                  {c}::/sy+:.{w}",
            "{w}   /                     {c}`--  /{w}",
            "{w}  `:                          {c}:`{w}",
            "{w}  `:                          {c}:`{w}",
            "{w}   /                          {c}/{w}",
            "{w}   .-                        {c}-.{w}",
            "{w}    --                      {c}-.{w}",
            "{w}     `:`                  {c}`:`{w}",
            "{w}       .--             `--.",
            "{w}          .---.....----.",
        ],
        distro_color: "\x1b[38;5;196m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "slackware",
        raw_lines: &[
            "{w}                  ::::::: ",
            "{w}            ::::::::::::::::::: ",
            "{w}         ::::::::::::::::::::::::: ",
            "{w}       ::::::::{c}cllcccccllllllll{w}:::::: ",
            "{w}    :::::::::{c}lc               dc{w}::::::: ",
            "{w}   ::::::::{c}cl   clllccllll    oc{w}::::::::: ",
            "{w}  :::::::::{c}o   lc{w}::::::::{c}co   oc{w}:::::::::: ",
            "{w} ::::::::::{c}o    cccclc{w}:::::{c}clcc{w}:::::::::::: ",
            "{w} :::::::::::{c}lc        cclccclc{w}::::::::::::: ",
            "{w}::::::::::::::{c}lcclcc          lc{w}:::::::::::: ",
            "{w}::::::::::{c}cclcc{w}:::::{c}lccclc     oc{w}::::::::::: ",
            "{w}::::::::::{c}o    l{w}::::::::::{c}l    lc{w}::::::::::: ",
            "{w} :::::{c}cll{w}:{c}o     clcllcccll     o{w}::::::::::: ",
            "{w} :::::{c}occ{w}:{c}o                  clc{w}::::::::::: ",
            "{w}  ::::{c}ocl{w}:{c}ccslclccclclccclclc{w}::::::::::::: ",
            "{w}   :::{c}oclcccccccccccccllllllllllllll{w}::::: ",
            "{w}    ::{c}lcc1lcccccccccccccccccccccccco{w}:::: ",
            "{w}      :::::::::::::::::::::::::::::::: ",
            "{w}        :::::::::::::::::::::::::::: ",
            "{w}           :::::::::::::::::::::: ",
            "{w}                :::::::::::: ",
        ],
        distro_color: "\x1b[38;5;61m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "artix",
        raw_lines: &[
            "{w}                   '",
            "{w}                  'o'",
            "{w}                 'ooo'",
            "{w}                'ooxoo'",
            "{w}               'ooxxxoo'",
            "{w}              'oookkxxoo'",
            "{w}             'oiioxkkxxoo'",
            "{w}            ':;:iiiioxxxoo'",
            "{w}               `'.;::ioxxoo'",
            "{w}          '-.      `':;j{c}iooo{w}'",
            "{w}         'oooio-..     `'{c}i:io{w}'",
            "{w}        'ooooxxxxoio:,.   `'{c}-;'{w}",
            "{w}       'ooooxxxxxkkxoooIi:-.  `'{c}",
            "{w}      'ooooxxxxxkkkkxoiiiiiji'{c}",
            "{w}     'ooooxxxxxkxxoiiii:'`     .{c}i'",
            "{w}    'ooooxxxxxoi:::'`       .;i{c}oxo'",
            "{w}   'ooooxooi::'`         .:iii{c}xkxxo'",
            "{w}  'ooooi:'`                `'';{c}ioxxo'",
            "{w} 'i:'`                          ''{c}:io'",
            "{w}'`                                   `'",
        ],
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "zorin",
        raw_lines: &[
            "{w}        `osssssssssssssssssssso`",
            "{w}       .osssssssssssssssssssssso.",
            "{w}      .+oooooooooooooooooooooooo+.",
            "",
            "",
            "{c}  `::::::::::::::::::::::.         .:`",
            "{c} `+ssssssssssssssssss+:.`     `.:+ssso`",
            "{c}.ossssssssssssssso/.       `-+ossssssso.",
            "{c}ssssssssssssso/-`      `-/osssssssssssss",
            "{c}.ossssssso/-`      .-/ossssssssssssssso.",
            "{c} `+sss+:.      `.:+ssssssssssssssssss+`",
            "{c}  `:.         .::::::::::::::::::::::`",
            "",
            "",
            "{w}      .+oooooooooooooooooooooooo+.",
            "{w}       -osssssssssssssssssssssso-",
            "{w}        `osssssssssssssssssssso`",
        ],
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "windows11",
        raw_lines: &[
            "{w}                                ..,",
            "{w}                    ....,,:;+ccllll",
            "{w}      ...,,+:;  cllllllllllllllllll",
            "{c},cclllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}`'ccllllllllll  {c}lllllllllllllllllll",
            "{w}       `' \\*::  {c}:ccllllllllllllllll",
            "{w}                       ````''*::cll",
            "{w}                                 ``",
        ],
        distro_color: "\x1b[38;5;39m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "windows10",
        raw_lines: &[
            "{w}                                ..,",
            "{w}                    ....,,:;+ccllll",
            "{w}      ...,,+:;  cllllllllllllllllll",
            "{c},cclllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "{c}llllllllllllll  {w}lllllllllllllllllll",
            "",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}llllllllllllll  {c}lllllllllllllllllll",
            "{w}`'ccllllllllll  {c}lllllllllllllllllll",
            "{w}       `' \\*::  {c}:ccllllllllllllllll",
            "{w}                       ````''*::cll",
            "{w}                                 ``",
        ],
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "windows7",
        raw_lines: &[
            "{w}        ,.=:!!t3Z3z.,",
            "{w}       :tt:::tt333EE3",
            "{w}       Et:::ztt33EEEL{c} @Ee.,      ..,{w}",
            "{w}      ;tt:::tt333EE7{c} ;EEEEEEttttt33#{w}",
            "{w}     :Et:::zt333EEQ.{c} $EEEEEttttt33QL{w}",
            "{w}     it::::tt333EEF{c} @EEEEEEttttt33F{w}",
            "{w}    ;3=*^```\"*4EEV{c} :EEEEEEttttt33@.{w}",
            "{c}    ,.=::::!t=., {w}`{c} @EEEEEEtttz33QF",
            "{c}   ;::::::::zt33)   \"4EEEtttji3P*",
            "{c}  :t::::::::tt33.:Z3z..  `` ,..g.",
            "{c}  i::::::::zt33F AEEEtttt::::ztF",
            "{c} ;:::::::::t33V ;EEEttttt::::t3",
            "{c} E::::::::zt33L @EEEtttt::::z3F",
            "{c}{3=*^```\"*4E3) ;EEEtttt:::::tZ`",
            "{c}             ` :EEEEtttt::::z7",
            "{c}                 \"VEzjt:;;z>*`",
        ],
        distro_color: "\x1b[38;5;33m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "android",
        raw_lines: &[
            "{c}         -o          o-",
            "{c}          +hydNNNNdyh+",
            "{c}        +mMMMMMMMMMMMMm+",
            "{c}      `dMM{w}m:{c}NMMMMMMN{w}:m{c}MMd`",
            "{c}      hMMMMMMMMMMMMMMMMMMh",
            "{c}  ..  yyyyyyyyyyyyyyyyyyyy  ..",
            "{c}.mMMm`MMMMMMMMMMMMMMMMMMMM`mMMm.",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:",
            "{c}-MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM-",
            "{c} +yy+ MMMMMMMMMMMMMMMMMMMM +yy+",
            "{c}      MMMMMMMMMMMMMMMMMMMM",
            "{c}      MMMMMMMMMMMMMMMMMMMM",
            "{c}      /++MMMMh++++hMMMM++/",
            "{c}         MMMMo    oMMMM",
            "{c}         MMMMo    oMMMM",
            "{c}         ooss     ssoo",
        ],
        distro_color: "\x1b[38;5;118m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "macos",
        raw_lines: &[
            "{w}                    'c.",
            "{w}                 ,xNMM.",
            "{w}               .OMMMMo",
            "{w}               lMM\"",
            "{w}     .;loddo:.  .oa.       .looo:.",
            "{w}   {c}cKMMMMMMMMMMNWMMMM4eecl{w}dMMMMMMMMMx",
            "{w} {c}.KMMMMMMMMMMMMMMMMMMMMMMMWd{w}.kMMMMMMMMK",
            "{w}{c}wMMMMMMMMMMMMMMMMMMMMMMMMMM{w}mo  Local",
            "{w}{c}lMMMMMMMMMMMMMMMMMMMMMMMMMM{w}Mo",
            "{w}{c} kMMMMMMMMMMMMMMMMMMMMMMMM{w}K'",
            "{w}  {c}kMMMMMMMMMMMMMMMMMMMMMMd",
            "{w}   {c}'xMMMMMMMMMMMMMMMMMMd.",
            "{w}     {c}'xNMMMMMMMMMMMMMNk'",
            "{w}        {c}';okkxOkkko;'.",
        ],
        distro_color: "\x1b[38;5;250m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "openbsd",
        raw_lines: &[
            "{w}                                       _",
            "{w}                                      (_)",
            "{c}              |    .",
            "{c}          .   |L  /|   .          {w}_",
            "{c}      _ . |\\ _| \\--+._/| .       {w}(_)",
            "{c}     / ||\\| Y J  )   / |/| ./",
            "{c}    J  |)'( |        ` F`.'/        {w}_",
            "{c}  -<|  F         __     .-<        {w}(_)",
            "{c}    | /       .-'.`}   . {'-.",
            "{c}    + |      /  /`L   /  |  \\",
            "{c}     `\\       [ [  _  [  |  ]",
            "{c}       \\__     \\._/ \\_/__/ ./",
            "{c}          `---._____.-'",
        ],
        distro_color: "\x1b[38;5;220m",
        outer_color: WHITE_COLOR,
    },
    Logo {
        name: "netbsd",
        raw_lines: &[
            "{c} \\\\`-______,----__",
            "{c}  \\\\        __,---`._",
            "{c}   \\\\       `.____",
            "{c}    \\\\            `---.",
            "{c}     \\\\                \\\\",
            "{c}      \\\\                \\\\",
            "{c}       \\\\  {w}`-----.{c}       \\\\",
            "{c}        \\\\   {w}/\\\\__\\\\{c}        \\\\",
            "{c}         \\\\ {w}/  __ \\\\{c}        \\\\",
            "{c}          \\\\{w}/  /  \\\\ \\\\{c}        \\\\",
            "{c}           {w}/  /    \\\\ \\\\{c}        \\\\",
            "{c}          {w}/  /      \\\\ \\\\{c}        \\\\",
            "{c}         {w}/__/{c}        {w}\\\\__\\\\{c}        \\\\",
        ],
        distro_color: "\x1b[38;5;208m",
        outer_color: WHITE_COLOR,
    },
];

/// Resolves a matching `Logo` based on the detected OS string or user override.
pub fn match_logo(
    logo_override: Option<&str>,
    distro_id: &str,
    distro_like: &[String],
) -> Option<&'static Logo> {
    if let Some(name) = logo_override {
        let name_lower = name.to_lowercase();
        if name_lower == "none" || name_lower == "off" {
            return None;
        }
        for logo in ALL_LOGOS {
            if logo.name.eq_ignore_ascii_case(&name_lower) {
                return Some(logo);
            }
        }
        // Aliases
        if name_lower == "kkfetch" || name_lower == "rust" {
            return ALL_LOGOS.iter().find(|l| l.name == "ferris");
        }
        if name_lower == "mint" {
            return ALL_LOGOS.iter().find(|l| l.name == "linuxmint");
        }
        if name_lower == "suse" {
            return ALL_LOGOS.iter().find(|l| l.name == "opensuse");
        }
        if name_lower == "tux" || name_lower == "linux" {
            return ALL_LOGOS.iter().find(|l| l.name == "generic");
        }
        if name_lower == "win" || name_lower == "windows" {
            return ALL_LOGOS.iter().find(|l| l.name == "windows11");
        }
        if name_lower == "win11" {
            return ALL_LOGOS.iter().find(|l| l.name == "windows11");
        }
        if name_lower == "win10" {
            return ALL_LOGOS.iter().find(|l| l.name == "windows10");
        }
        if name_lower == "win7" {
            return ALL_LOGOS.iter().find(|l| l.name == "windows7");
        }
        if name_lower == "mac"
            || name_lower == "darwin"
            || name_lower == "apple"
            || name_lower == "osx"
        {
            return ALL_LOGOS.iter().find(|l| l.name == "macos");
        }
        if name_lower == "bsd" {
            return ALL_LOGOS.iter().find(|l| l.name == "freebsd");
        }
    }

    let id_clean = distro_id.to_lowercase().replace(' ', "");

    // 1. Direct exact match
    for logo in ALL_LOGOS {
        if id_clean == logo.name {
            return Some(logo);
        }
    }

    // 2. Specific OS & Distribution keyword matching
    if id_clean.contains("android") || id_clean.contains("termux") {
        return ALL_LOGOS.iter().find(|l| l.name == "android");
    }
    if id_clean.contains("macos")
        || id_clean.contains("darwin")
        || id_clean.contains("osx")
        || id_clean.contains("apple")
    {
        return ALL_LOGOS.iter().find(|l| l.name == "macos");
    }
    if id_clean.contains("freebsd") {
        return ALL_LOGOS.iter().find(|l| l.name == "freebsd");
    }
    if id_clean.contains("openbsd") {
        return ALL_LOGOS.iter().find(|l| l.name == "openbsd");
    }
    if id_clean.contains("netbsd") {
        return ALL_LOGOS.iter().find(|l| l.name == "netbsd");
    }
    if id_clean.contains("windows11") || id_clean.contains("win11") {
        return ALL_LOGOS.iter().find(|l| l.name == "windows11");
    }
    if id_clean.contains("windows10") || id_clean.contains("win10") {
        return ALL_LOGOS.iter().find(|l| l.name == "windows10");
    }
    if id_clean.contains("windows7") || id_clean.contains("win7") {
        return ALL_LOGOS.iter().find(|l| l.name == "windows7");
    }
    if id_clean.contains("windows") || id_clean.contains("win") {
        return ALL_LOGOS.iter().find(|l| l.name == "windows11");
    }
    if id_clean.contains("ubuntu") {
        return ALL_LOGOS.iter().find(|l| l.name == "ubuntu");
    }
    if id_clean.contains("mint") || id_clean == "linuxmint" {
        return ALL_LOGOS.iter().find(|l| l.name == "linuxmint");
    }
    if id_clean.contains("fedora") {
        return ALL_LOGOS.iter().find(|l| l.name == "fedora");
    }
    if id_clean.contains("endeavour") {
        return ALL_LOGOS.iter().find(|l| l.name == "endeavouros");
    }
    if id_clean.contains("manjaro") {
        return ALL_LOGOS.iter().find(|l| l.name == "manjaro");
    }
    if id_clean.contains("artix") {
        return ALL_LOGOS.iter().find(|l| l.name == "artix");
    }
    if id_clean.contains("arch") {
        return ALL_LOGOS.iter().find(|l| l.name == "arch");
    }
    if id_clean.contains("debian") {
        return ALL_LOGOS.iter().find(|l| l.name == "debian");
    }
    if id_clean.contains("redhat") || id_clean.contains("rhel") || id_clean.contains("centos") {
        return ALL_LOGOS.iter().find(|l| l.name == "rhel");
    }
    if id_clean.contains("rocky") {
        return ALL_LOGOS.iter().find(|l| l.name == "rocky");
    }
    if id_clean.contains("alma") {
        return ALL_LOGOS.iter().find(|l| l.name == "almalinux");
    }
    if id_clean.contains("suse") || id_clean.contains("opensuse") {
        return ALL_LOGOS.iter().find(|l| l.name == "opensuse");
    }
    if id_clean.contains("gentoo") {
        return ALL_LOGOS.iter().find(|l| l.name == "gentoo");
    }
    if id_clean.contains("alpine") {
        return ALL_LOGOS.iter().find(|l| l.name == "alpine");
    }
    if id_clean.contains("void") {
        return ALL_LOGOS.iter().find(|l| l.name == "void");
    }
    if id_clean.contains("pop") {
        return ALL_LOGOS.iter().find(|l| l.name == "pop");
    }
    if id_clean.contains("nix") {
        return ALL_LOGOS.iter().find(|l| l.name == "nixos");
    }
    if id_clean.contains("kali") {
        return ALL_LOGOS.iter().find(|l| l.name == "kali");
    }
    if id_clean.contains("slackware") {
        return ALL_LOGOS.iter().find(|l| l.name == "slackware");
    }
    if id_clean.contains("zorin") {
        return ALL_LOGOS.iter().find(|l| l.name == "zorin");
    }

    // 3. Parent ID_LIKE fallbacks
    for like in distro_like {
        let like_lower = like.to_lowercase();
        if like_lower.contains("android") {
            return ALL_LOGOS.iter().find(|l| l.name == "android");
        }
        if like_lower.contains("ubuntu") {
            return ALL_LOGOS.iter().find(|l| l.name == "ubuntu");
        }
        if like_lower.contains("debian") {
            return ALL_LOGOS.iter().find(|l| l.name == "debian");
        }
        if like_lower.contains("arch") {
            return ALL_LOGOS.iter().find(|l| l.name == "arch");
        }
        if like_lower.contains("fedora") {
            return ALL_LOGOS.iter().find(|l| l.name == "fedora");
        }
        if like_lower.contains("rhel") {
            return ALL_LOGOS.iter().find(|l| l.name == "rhel");
        }
        if like_lower.contains("suse") {
            return ALL_LOGOS.iter().find(|l| l.name == "opensuse");
        }
    }

    // Default fallback to generic Linux or ferris crab
    ALL_LOGOS
        .iter()
        .find(|l| l.name == "generic")
        .or_else(|| ALL_LOGOS.iter().find(|l| l.name == "ferris"))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_match_logo_direct() {
        let logo = match_logo(Some("ubuntu"), "ubuntu", &[]).unwrap();
        assert_eq!(logo.name, "ubuntu");
    }

    #[test]
    fn test_match_logo_like_fallback() {
        let logo = match_logo(None, "my_custom_distro", &["ubuntu".to_string()]).unwrap();
        assert_eq!(logo.name, "ubuntu");
    }

    #[test]
    fn test_match_logo_none_override() {
        assert!(match_logo(Some("none"), "ubuntu", &[]).is_none());
        assert!(match_logo(Some("off"), "ubuntu", &[]).is_none());
    }

    #[test]
    fn test_match_logo_override() {
        let logo = match_logo(Some("arch"), "ubuntu", &[]).unwrap();
        assert_eq!(logo.name, "arch");
    }

    #[test]
    fn test_match_logo_unknown_fallback_to_ferris() {
        let logo = match_logo(None, "unknown_distro", &[]).unwrap();
        assert_eq!(logo.name, "generic");
    }

    #[test]
    fn test_match_logo_kkfetch_and_ferris_aliases() {
        let logo_kk = match_logo(Some("kkfetch"), "generic", &[]).unwrap();
        assert_eq!(logo_kk.name, "ferris");
        let logo_rust = match_logo(Some("rust"), "generic", &[]).unwrap();
        assert_eq!(logo_rust.name, "ferris");
    }

    #[test]
    fn test_match_logo_windows() {
        let logo = match_logo(None, "windows 11", &[]).unwrap();
        assert_eq!(logo.name, "windows11");
        let logo_win10 = match_logo(None, "windows 10", &[]).unwrap();
        assert_eq!(logo_win10.name, "windows10");
        let logo_win7 = match_logo(None, "windows 7", &[]).unwrap();
        assert_eq!(logo_win7.name, "windows7");
    }

    #[test]
    fn test_match_logo_android() {
        let logo = match_logo(None, "android", &[]).unwrap();
        assert_eq!(logo.name, "android");
        let logo_termux = match_logo(None, "termux", &[]).unwrap();
        assert_eq!(logo.name, "android");
        assert_eq!(logo_termux.name, "android");
    }

    #[test]
    fn test_match_logo_macos_and_bsd() {
        let logo_mac = match_logo(None, "macos", &[]).unwrap();
        assert_eq!(logo_mac.name, "macos");
        let logo_darwin = match_logo(None, "darwin", &[]).unwrap();
        assert_eq!(logo_darwin.name, "macos");

        let logo_freebsd = match_logo(None, "freebsd", &[]).unwrap();
        assert_eq!(logo_freebsd.name, "freebsd");
        let logo_openbsd = match_logo(None, "openbsd", &[]).unwrap();
        assert_eq!(logo_openbsd.name, "openbsd");
        let logo_netbsd = match_logo(None, "netbsd", &[]).unwrap();
        assert_eq!(logo_netbsd.name, "netbsd");
    }

    #[test]
    fn test_match_logo_linux_generic_does_not_match_mint() {
        let logo = match_logo(None, "linux", &[]).unwrap();
        assert_eq!(logo.name, "generic");
        assert_ne!(logo.name, "linuxmint");
    }
}
