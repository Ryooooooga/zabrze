zle -N __zabrze::expand
zle -N __zabrze::expand-and-insert-self
zle -N __zabrze::expand-and-accept-line
zle -N __zabrze::insert-space

__zabrze::expand() {
    local out exit_code
    out="$(zabrze expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")"
    exit_code="$?"
    [ "$exit_code" -eq 0 ] && eval "$out"
}

__zabrze::expand-and-insert-self() {
    zle __zabrze::expand
    zle self-insert
}

__zabrze::expand-and-accept-line() {
    zle __zabrze::expand
    zle reset-prompt
    zle accept-line
}

__zabrze::insert-space() {
    LBUFFER+=" "
}
