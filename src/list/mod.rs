use crate::config::{Config, Trigger};
use crate::opt::ListArgs;
use shell_escape::escape;
use std::borrow::Cow;
use std::io;

pub fn run(args: &ListArgs) {
    list(args, &Config::load_or_exit(), &mut io::stdout()).unwrap();
}

fn list<W: io::Write>(_args: &ListArgs, config: &Config, out: &mut W) -> Result<(), io::Error> {
    for abbrev in &config.abbrevs {
        let abbr = match &abbrev.trigger {
            Trigger::Abbr(abbr) => abbr,
            Trigger::Regex(regex) => regex,
        };
        let snippet = escape(Cow::from(&abbrev.snippet));

        writeln!(out, "{}={}", abbr, snippet)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config::load_from_str(
            r"
            abbrevs:
              - name: git
                abbr: g
                snippet: git

              - name: git commit
                abbr: c
                snippet: commit
                global: true
                context: '^git '

              - name: '>/dev/null'
                abbr: 'null'
                snippet: '>/dev/null'
                global: true

              - name: $HOME
                abbr: home
                snippet: $HOME
                evaluate: true

              - name: ..
                abbr: ^\.\.(/\.\.)*$
                snippet: cd
                action: prepend
            ",
        )
        .unwrap()
    }

    #[test]
    fn test_list() {
        let args = ListArgs {};
        let config = test_config();

        let mut buf = Vec::new();
        list(&args, &config, &mut std::io::BufWriter::new(&mut buf)).unwrap();

        let output = std::str::from_utf8(&buf).unwrap();

        let expected = r"g=git
c=commit
null='>/dev/null'
home='$HOME'
^\.\.(/\.\.)*$=cd
";

        assert_eq!(output, expected);
    }
}
