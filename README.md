# zabrze

[![](https://github.com/Ryooooooga/zabrze/actions/workflows/build.yml/badge.svg)](https://github.com/Ryooooooga/zabrze/actions/workflows/build.yml)
[![](https://badgen.net/crates/v/zabrze)](https://crates.io/crates/zabrze)

ZSH abbreviation expansion plugin

## Usage

### Simple abbreviation

```yaml
# ~/.config/zabrze/config.yaml
abbrevs:
  - name: git
    abbr: g
    snippet: git

  - name: awk '{print $1}'
    abbr: '.1'
    snippet: awk '{print $1}'
```

```zsh
$ eval "$(zabrze init --bind-keys)"
```

then

```zsh
$ g<SP>
#  ↓ expanded
$ git

$ cat a.txt | .1<CR>
#  ↓ expanded and executed
$ cat a.txt | awk '{print $1}'
```

### Global abbreviation

```yaml
abbrevs:
  - name: '>/dev/null 2>&1'
    abbr: 'null'
    snippets: '>/dev/null 2>&1'
    global: true
```

```zsh
$ echo a null<SP>
#  ↓ expanded
$ echo a >/dev/null 2>&1
```

### Global abbreviation with context

```yaml
abbrevs:
  - name: git commit
    abbr: c
    snippet: commit
    global: true
    context: '^git\s'
    
  - name: git commit -m
    abbr: cm
    snippet: commit -m '{}'
    cursor: "{}" # optional; defaults to "{}"
    global: true
    context: '^git\s'

  - name: branch name
    abbr: B
    snippet: $(git symbolic-ref --short HEAD)
    evaluate: true
    global: true
    context: '^git\s'
```

```zsh
$ git c<SP>
#  ↓ expanded
$ git commit

$ git cm<SP>
#  ↓ expanded and move into quotes
$ git commit -m '|'

$ git push -d origin B<CR>
#  ↓ expanded and executed
$ git push -d origin main
```

### Conditional abbreviation

```yaml
abbrevs:
  - name: chrome
    abbr: chrome
    snippet: open -a 'Google Chrome'
    if: '[[ "$OSTYPE" =~ darwin ]]' # only available in macOS

  - name: trash
    abbr: rm
    snippet: trash
    if: (( ${+commands[trash]} )) # available if trash is installed

  - name: rm -r
    abbr: rm
    snippet: rm -r # fallback
```

### Suffix alias

```yaml
abbrevs:
  - name: python3 *.py
    abbr-pattern: ^(?<file>.+\.py)$
    snippet: python3 $file
    evaluate: true

  # or
  - name: python3 *.py
    abbr-pattern: \.py$
    snippet: python3 $abbr
    evaluate: true
```

```zsh
$ ./a.py<CR>
#  ↓ expanded and executed
$ python3 ./a.py
```

## Installation

### From prebuilt binary

You can download a binary release [here](https://github.com/Ryooooooga/zabrze/releases).

### zinit

```zsh
zinit blockf light-mode as"program" from"gh-r" for \
    atload'eval "$(zabrze init --bind-keys)"' \
    Ryooooooga/zabrze
```

### Cargo

```zsh
$ cargo install zabrze
```

### Homebrew

```zsh
$ brew install ryooooooga/tap/zabrze
```

## Configuration

zabrze reads configuration files from the following locations:

- `$ZABRZE_CONFIG_HOME` if set, otherwise `$XDG_CONFIG_HOME/zabrze` (defaults to `$HOME/.config/zabrze`)
- Configuration files are read in lexicographical order.
- Supported file extensions are `yaml` and `yml`.

The configuration file is a YAML file that defines a list of abbreviations. Each abbreviation has the following properties:

- `name` (string): A descriptive name for the abbreviation.
- `abbr` (string, required, mutually exclusive with `abbr-pattern`): The abbreviation to expand.
- `abbr-pattern` (string, required, mutually exclusive with `abbr`): A regular expression to match the abbreviation.
- `snippet` (string, required): The text to replace the abbreviation with.
- `global` (boolean): A boolean value indicating whether the abbreviation should be expanded globally. Defaults to `false`.
- `context` (string): A regular expression that must match the beginning of the line for the abbreviation to be expanded.
- `evaluate` (boolean): A boolean value indicating whether the snippet should be evaluated as a shell command. Defaults to `false`.
- `if` (string): A conditional expression that must evaluate to true for the abbreviation to be expanded.
- `cursor` (string or `null`): A string that specifies the cursor position after expansion. Defaults to `{}`.

## Alternatives

- [zsh-abbrev-alias](https://github.com/momo-lab/zsh-abbrev-alias)
- [zeno.zsh](https://github.com/yuki-yano/zeno.zsh)
