# Bash completion for annactl
# Install to: /usr/share/bash-completion/completions/annactl

_annactl() {
    local cur prev words cword
    _init_completion || return

    local commands="ping doctor status config help"
    local config_actions="get set list"
    local config_scopes="user system"
    local config_keys="autonomy.level telemetry.local_store shell.integrations.autocomplete daemon.socket_path daemon.pid_file logging.level logging.directory"

    # First level: main commands
    if [[ $cword -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
        return
    fi

    # Second level: based on previous command
    case "${words[1]}" in
        config)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$config_actions" -- "$cur") )
            elif [[ $cword -eq 3 ]]; then
                case "${words[2]}" in
                    get)
                        COMPREPLY=( $(compgen -W "$config_keys" -- "$cur") )
                        ;;
                    set)
                        COMPREPLY=( $(compgen -W "$config_scopes" -- "$cur") )
                        ;;
                esac
            elif [[ $cword -eq 4 && "${words[2]}" == "set" ]]; then
                COMPREPLY=( $(compgen -W "$config_keys" -- "$cur") )
            fi
            ;;
        *)
            ;;
    esac
}

complete -F _annactl annactl
