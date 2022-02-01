use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct ExpandResult<'a> {
    pub lbuffer: &'a str,
    pub rbuffer: &'a str,
    pub left_snippet: &'a str,
    pub right_snippet: &'a str,
    pub evaluate: bool,
    pub insert_space: bool,
}

pub fn run(args: &ExpandArgs) {
    if let Some(result) = expand(args, &Config::load_or_exit()) {
        let lbuffer = escape(Cow::from(result.lbuffer));
        let left_snippet = escape(Cow::from(result.left_snippet));
        let right_snippet = escape(Cow::from(result.right_snippet));
        let rbuffer = escape(Cow::from(result.rbuffer));
        let evaluate = if result.evaluate { "(e)" } else { "" };
        let insert_space = if result.insert_space { "1" } else { "" };

        println!(
            r#"local lbuffer={lbuffer};local rbuffer={rbuffer};local left_snippet={left_snippet};local right_snippet={right_snippet};LBUFFER="${{lbuffer}}${{{evaluate}left_snippet}}";RBUFFER="${{{evaluate}right_snippet}}${{rbuffer}}";__zabrze_insert_space={insert_space}"#,
            lbuffer = lbuffer,
            rbuffer = rbuffer,
            left_snippet = left_snippet,
            right_snippet = right_snippet,
            evaluate = evaluate,
            insert_space = insert_space,
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

    let (left_snippet, right_snippet, insert_space) = split_snippet(&abbrev.snippet);

    Some(ExpandResult {
        lbuffer: lbuffer_without_last_arg,
        rbuffer,
        left_snippet,
        right_snippet,
        evaluate: abbrev.evaluate,
        insert_space,
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

              - name: git commit -m ''
                abbr: cm
                snippet: commit -m '{}'
                global: true
                context: '^git '
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
                    lbuffer: "",
                    rbuffer: "",
                    left_snippet: "git",
                    right_snippet: "",
                    evaluate: false,
                    insert_space: true,
                }),
            },
            Scenario {
                testname: "simple abbr with rbuffer",
                lbuffer: "g",
                rbuffer: " --pager=never",
                expected: Some(ExpandResult {
                    lbuffer: "",
                    rbuffer: " --pager=never",
                    left_snippet: "git",
                    right_snippet: "",
                    evaluate: false,
                    insert_space: true,
                }),
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; ",
                    rbuffer: "",
                    left_snippet: "git",
                    right_snippet: "",
                    evaluate: false,
                    insert_space: true,
                }),
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello ",
                    rbuffer: "",
                    left_snippet: ">/dev/null",
                    right_snippet: "",
                    evaluate: false,
                    insert_space: true,
                }),
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                rbuffer: " -m hello",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; git ",
                    rbuffer: " -m hello",
                    left_snippet: "commit",
                    right_snippet: "",
                    evaluate: false,
                    insert_space: true,
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
                    lbuffer: "",
                    rbuffer: "",
                    left_snippet: "$HOME",
                    right_snippet: "",
                    evaluate: true,
                    insert_space: true,
                }),
            },
            Scenario {
                testname: "simple abbr with placeholder",
                lbuffer: "git cm",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "git ",
                    rbuffer: "",
                    left_snippet: "commit -m '",
                    right_snippet: "'",
                    evaluate: false,
                    insert_space: false,
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

fn split_snippet(snippet: &str) -> (&str, &str, bool) {
    const PLACEHOLDER: &str = "{}";
    snippet
        .split_once(PLACEHOLDER)
        .map(|(left, right)| (left, right, false))
        .unwrap_or((snippet, "", true))
}

#[test]
fn test_split_snippet() {
    assert_eq!(split_snippet("foo bar"), ("foo bar", "", true));
    assert_eq!(split_snippet("foo{}bar"), ("foo", "bar", false));
}
