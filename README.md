# zabrze

zsh abbreviation exapnsion plugin

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
```

```zsh
$ eval "$(zabrze init --bind-keys)"
```

then

```zsh
$ g<SP>cm<SP>
#  ↓ expanded
$ git commit -m 
```

## Alternatives

- [zsh-abbrev-alias](https://github.com/momo-lab/zsh-abbrev-alias)
- [zeno.zsh](https://github.com/yuki-yano/zeno.zsh)
