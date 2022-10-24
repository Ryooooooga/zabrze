zle -N __zabrze::expand
zle -N __zabrze::expand-and-self-insert
zle -N __zabrze::expand-and-accept-line
zle -N __zabrze::insert-space

__zabrze::expand() {
    local out exit_code
    out="$(zabrze expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")"
    exit_code="$?"
    if [[ "$exit_code" -eq 0 ]] && [[ -n "$out" ]]; then
        eval "$out"
        if [[ -n "$ZABRZE_LOG_PATH" ]]; then
            \command mkdir -p "${ZABRZE_LOG_PATH:a:h}"
            \builtin printf "expand\t%s\t%s\n" "$EPOCHSECONDS" "$name" >> "$ZABRZE_LOG_PATH"
        fi
    fi
}

__zabrze::expand-and-self-insert() {
    zle __zabrze::expand
    zle reset-prompt
    [[ -z "$__zabrze_has_placeholder" ]] && zle self-insert
    unset __zabrze_has_placeholder
}

__zabrze::expand-and-accept-line() {
    zle __zabrze::expand
    zle reset-prompt
    zle accept-line
}

__zabrze::insert-space() {
    LBUFFER+=" "
}
