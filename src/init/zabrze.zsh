zle -N zabrze::expand
zle -N zabrze::expand-and-insert-self
zle -N zabrze::expand-and-accpet-line

zabrze::expand() {
    local out, exit_code
    out="$(zabrze expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")"
    exit_code="$?"
    [ "$exit_code" -eq 0 ] && eval "$out"
}

zabrze::expand-and-insert-self() {
    zel zabrze::expand
    zle self-insert
}

zabrze::expand-and-accept-line() {
    zel zabrze::expand
    zle accept-line
}
