use crate::config::{Config, Trigger};
use crate::opt::ListArgs;
use shell_escape::escape;
use std::borrow::Cow;
use std::io;

pub fn run(args: &ListArgs) {
    list(args, &Config::load_or_exit(), &mut io::stdout()).unwrap();
}

fn list<W: io::Write>(_args: &ListArgs, config: &Config, out: &mut W) -> Result<(), io::Error> {
    for snippet in &config.snippets {
        let trigger = match &snippet.trigger {
            Trigger::Text(text) => text,
            Trigger::Regex(regex) => regex,
        };
        let snippet = escape(Cow::from(&snippet.snippet));

        writeln!(out, "{}={}", trigger, snippet)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config::load_from_str(
            r#"
            [[snippets]]
            trigger = "g"
            snippet = "git"

            [[snippets]]
            name = "git commit"
            trigger = "c"
            snippet = "commit"
            global = true
            context = "^git "

            [[snippets]]
            name = ">/dev/null"
            trigger = "null"
            snippet = ">/dev/null"
            global = true

            [[snippets]]
            name = "$HOME"
            trigger = "home"
            snippet = "$HOME"
            evaluate = true

            [[snippets]]
            name = ".."
            trigger-pattern = '^\.\.(/\.\.)*$'
            snippet = "cd $trigger"
            evaluate = true
            "#,
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
^\.\.(/\.\.)*$='cd $trigger'
";

        assert_eq!(output, expected);
    }
}
