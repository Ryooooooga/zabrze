zle -N zabrze::expand
zle -N zabrze::expand-and-insert-self
zle -N zabrze::expand-and-accept-line
zle -N zabrze::insert-space

zabrze::expand() {
    local out exit_code
    out="$(zabrze expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")"
    exit_code="$?"
    [ "$exit_code" -eq 0 ] && eval "$out"
}

zabrze::expand-and-insert-self() {
    zle zabrze::expand
    zle self-insert
}

zabrze::expand-and-accept-line() {
    zle zabrze::expand
    zle accept-line
}

zabrze::insert-space() {
    LBUFFER+=" "
}
