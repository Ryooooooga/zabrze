use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct ExpandResult<'a> {
    pub command: &'a str,
    pub snippet: &'a str,
    pub evaluate: bool,
    pub rbuffer: &'a str,
}

pub fn run(args: &ExpandArgs) {
    if let Some(result) = expand(args, &Config::load_or_exit()) {
        let command = escape(Cow::from(result.command));
        let snippet = escape(Cow::from(result.snippet));
        let rbuffer = escape(Cow::from(result.rbuffer));
        let evaluate = if result.evaluate { "(e)" } else { "" };

        println!(
            r#"local command={};local snippet={};LBUFFER="${{command}}${{{}snippet}}";RBUFFER={};"#,
            command, snippet, evaluate, rbuffer
        );
    }
}

fn expand<'a>(args: &'a ExpandArgs, config: &'a Config) -> Option<ExpandResult<'a>> {
    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let command_index = find_last_command_index(lbuffer);
    let command = lbuffer[command_index..].trim_start();

    let (_, last_arg) = command
        .rsplit_once(char::is_whitespace)
        .unwrap_or(("", command));

    let abbrev = config
        .abbrevs
        .iter()
        .find(|abbr| abbr.is_match(command, last_arg))?;

    let last_arg_index = lbuffer.len() - last_arg.len();
    let lbuffer_without_last_arg = &lbuffer[..last_arg_index];

    Some(ExpandResult {
        command: lbuffer_without_last_arg,
        snippet: &abbrev.snippet,
        evaluate: abbrev.evaluate,
        rbuffer,
    })
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
                abbr: null
                snippet: '>/dev/null'
                global: true

              - name: $HOME
                abbr: home
                snippet: $HOME
                evaluate: true
            ",
        )
        .unwrap()
    }

    #[test]
    fn test_expand() {
        let config = test_config();

        struct Scenario<'a> {
            pub testname: &'a str,
            pub lbuffer: &'a str,
            pub rbuffer: &'a str,
            pub expected: Option<ExpandResult<'a>>,
        }

        let scenarios = &[
            Scenario {
                testname: "empty",
                lbuffer: "",
                rbuffer: "",
                expected: None,
            },
            Scenario {
                testname: "simple abbr",
                lbuffer: "g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    command: "",
                    snippet: "git",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "simple abbr with rbuffer",
                lbuffer: "g",
                rbuffer: " --pager=never",
                expected: Some(ExpandResult {
                    command: "",
                    snippet: "git",
                    evaluate: false,
                    rbuffer: " --pager=never",
                }),
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    command: "echo hello; ",
                    snippet: "git",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                rbuffer: "",
                expected: Some(ExpandResult {
                    command: "echo hello ",
                    snippet: ">/dev/null",
                    evaluate: false,
                    rbuffer: "",
                }),
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                rbuffer: " -m hello",
                expected: Some(ExpandResult {
                    command: "echo hello; git ",
                    snippet: "commit",
                    evaluate: false,
                    rbuffer: " -m hello",
                }),
            },
            Scenario {
                testname: "global abbr with miss matched context",
                lbuffer: "echo git c",
                rbuffer: "",
                expected: None,
            },
            Scenario {
                testname: "no matched abbr",
                lbuffer: "echo",
                rbuffer: " hello",
                expected: None,
            },
            Scenario {
                testname: "simple abbr with evaluate=true",
                lbuffer: "home",
                rbuffer: "",
                expected: Some(ExpandResult {
                    command: "",
                    snippet: "$HOME",
                    evaluate: true,
                    rbuffer: "",
                }),
            },
        ];

        for s in scenarios {
            let args = ExpandArgs {
                lbuffer: s.lbuffer.to_string(),
                rbuffer: s.rbuffer.to_string(),
            };

            let actual = expand(&args, &config);

            assert_eq!(actual, s.expected, "{}", s.testname);
        }
    }
}

fn find_last_command_index(line: &str) -> usize {
    line.rfind(|c| matches!(c, ';' | '&' | '|' | '(' | '`' | '\n'))
        .map(|i| i + 1)
        .unwrap_or(0)
}

#[test]
fn test_find_last_command_index() {
    assert_eq!(find_last_command_index("git commit"), 0);
    assert_eq!(find_last_command_index("echo hello; git commit"), 11);
    assert_eq!(find_last_command_index("echo hello && git commit"), 13);
    assert_eq!(find_last_command_index("seq 10 | tail -3 | cat"), 18);
}
