zle -N zabrze::expand
zle -N zabrze::expand-and-insert-self
zle -N zabrze::expand-and-accpet-line

zabrze::expand() {
    eval "$(zabrze expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")"
}

zabrze::expand-and-insert-self() {
    zel zabrze::expand
    zle self-insert
}

zabrze::expand-and-accept-line() {
    zel zabrze::expand
    zle accept-line
}
