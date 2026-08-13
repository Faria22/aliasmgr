autoload -Uz add-zle-hook-widget

_aliasmgr_vhs_highlight_command() {
    region_highlight=()

    local -a words
    words=(${(z)BUFFER})
    local command_name=${words[1]-}

    if [[ -n $command_name ]] &&
        (( $+commands[$command_name] ||
            $+builtins[$command_name] ||
            $+aliases[$command_name] ||
            $+functions[$command_name] ||
            $+reswords[$command_name] )); then
        region_highlight=("0 ${#command_name} fg=green")
    fi
}

add-zle-hook-widget line-pre-redraw _aliasmgr_vhs_highlight_command
