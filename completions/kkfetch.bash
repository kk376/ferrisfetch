# Bash completion script for kkfetch

_kkfetch() {
    local cur prev words cword
    if declare -F _init_completion >/dev/null 2>&1; then
        _init_completion -n : 2>/dev/null
    else
        if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
            cur="$2"
        else
            cur="${COMP_WORDS[COMP_CWORD]}"
        fi
        prev="$3"
    fi

    local opts="-m -d -l -h -V --modules --disable --no-color --logo --no-logo --list-modules --disk-path --json --help --version"
    local modules="title os host kernel installed uptime packages pkgs shell display desktop de wm terminal term cpu gpu memory mem swap disk battery localip theme icons colors palette"
    local logos="ferris rust debian ubuntu linuxmint mint fedora arch archlinux rhel redhat centos rocky rockylinux almalinux alma endeavouros endeavour manjaro generic tux linux opensuse suse alpine gentoo void pop popos nixos kali freebsd slackware artix zorin none"

    case "${prev}" in
        -m|--modules|-d|--disable)
            local prefix=""
            local item="$cur"
            if [[ "$cur" == *,* ]]; then
                prefix="${cur%,*},"
                item="${cur##*,}"
            fi
            local matches
            matches=$(compgen -W "$modules" -- "$item")
            COMPREPLY=()
            for m in $matches; do
                COMPREPLY+=("${prefix}${m}")
            done
            return 0
            ;;
        -l|--logo)
            COMPREPLY=( $(compgen -W "$logos" -- "$cur") )
            return 0
            ;;
        --disk-path)
            COMPREPLY=( $(compgen -d -- "$cur") )
            return 0
            ;;
    esac

    if [[ "$cur" == -* || ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
        return 0
    fi
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _kkfetch -o nosort -o bashdefault -o default kkfetch
else
    complete -F _kkfetch -o bashdefault -o default kkfetch
fi
