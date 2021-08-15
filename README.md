# zabrze

[![](https://github.com/Ryooooooga/zabrze/actions/workflows/build.yml/badge.svg)](https://github.com/Ryooooooga/zabrze/actions/workflows/build.yml)
[![](https://badgen.net/crates/v/zabrze)](https://crates.io/crates/zabrze)

ZSH abbreviation expansion plugin

## Usage

```yaml
# ~/.config/zabrze/config.yaml
abbrevs:
  # abbrev alias
  - name: git
    abbr: g
    snippet: git

  # global abbrev
  - name: '>/dev/null'
    abbr: null
    snippets: '>/dev/null'
    global: true

  # global abbrev with context
  - name: git commit -m
    abbr: cm
    snippet: commit -m
    global: true
    context: '^git\s+'

  - name: branch name
    abbr: B
    snippet: $(git symbolic-ref --short HEAD)
    evaluate: true
    global: true
    context: '^git\s+'
```

```zsh
$ eval "$(zabrze init --bind-keys)"
```

then

```zsh
$ g<SP>cm<SP>
#  ↓ expanded
$ git commit -m 

$ git show B<CR>
#  ↓ expanded
$ git show main
```

## Installation

### From prebuilt binary

You can download a binary release [here](https://github.com/Ryooooooga/zabrze/releases).

## zinit

```zsh
zinit blockf light-mode as"program" from"gh-r" for \
    atload'eval "$(zabrze init --bind-keys)"' \
    Ryooooooga/zabrze
```

## Cargo

```zsh
$ cargo install zabrze
```

## Alternatives

- [zsh-abbrev-alias](https://github.com/momo-lab/zsh-abbrev-alias)
- [zeno.zsh](https://github.com/yuki-yano/zeno.zsh)
