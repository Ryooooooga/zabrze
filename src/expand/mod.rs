use crate::config::{Action, Config};
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct ExpandResult<'a> {
    pub lbuffer: &'a str,
    pub rbuffer: &'a str,
    pub replacement: SnippetReplacement<'a>,
    pub left_snippet: &'a str,
    pub right_snippet: &'a str,
    pub evaluate: bool,
    pub has_placeholder: bool,
}

#[derive(Debug, PartialEq)]
pub struct SnippetReplacement<'a> {
    pub start_index: usize,
    pub end_index: usize,
    pub snippet_prefix: &'a str,
    pub snippet_suffix: &'a str,
}

pub fn run(args: &ExpandArgs) {
    if let Some(result) = expand(args, &Config::load_or_exit()) {
        let snippet_start_index = result.replacement.start_index;
        let snippet_end_index = result.replacement.end_index;

        let lbuffer_pre = escape(Cow::from(&result.lbuffer[..snippet_start_index]));
        let lbuffer_post = escape(Cow::from(&result.lbuffer[snippet_end_index..]));
        let snippet_prefix = escape(Cow::from(result.replacement.snippet_prefix));
        let snippet_suffix = escape(Cow::from(result.replacement.snippet_suffix));
        let left_snippet = escape(Cow::from(result.left_snippet));
        let right_snippet = escape(Cow::from(result.right_snippet));

        let rbuffer = escape(Cow::from(result.rbuffer));
        let evaluate = if result.evaluate { "(e)" } else { "" };

        print!(r"local lbuffer_pre={lbuffer_pre}{snippet_prefix};");
        print!(r"local lbuffer_post={snippet_suffix}{lbuffer_post};");
        print!(r"local rbuffer={rbuffer};");
        print!(r"local left_snippet={left_snippet};");
        if result.has_placeholder {
            print!(r"local right_snippet={right_snippet};");
            print!(r#"LBUFFER="${{lbuffer_pre}}${{{evaluate}left_snippet}}";"#);
            print!(r#"RBUFFER="${{{evaluate}right_snippet}}${{lbuffer_post}}${{rbuffer}}";"#);
            print!(r"__zabrze_has_placeholder=1;");
        } else {
            print!(r#"LBUFFER="${{lbuffer_pre}}${{{evaluate}left_snippet}}${{lbuffer_post}}";"#);
            print!(r#"RBUFFER="${{rbuffer}}";"#);
            print!(r"__zabrze_has_placeholder=;");
        }
        println!();
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

    if last_arg.is_empty() {
        return None;
    }

    let matched = config
        .abbrevs
        .iter()
        .flat_map(|abbr| abbr.do_match(command, last_arg))
        .next()?;

    let replacement = match matched.action() {
        Action::ReplaceLast => {
            let last_arg_start_index = lbuffer.len() - last_arg.len();
            let last_arg_end_index = lbuffer.len();
            SnippetReplacement {
                start_index: last_arg_start_index,
                end_index: last_arg_end_index,
                snippet_prefix: "",
                snippet_suffix: "",
            }
        }
        Action::ReplaceAll => {
            let command_start_index = lbuffer.len() - command.len();
            let command_end_index = lbuffer.len();
            SnippetReplacement {
                start_index: command_start_index,
                end_index: command_end_index,
                snippet_prefix: "",
                snippet_suffix: "",
            }
        }
        Action::Prepend => {
            let command_start_index = lbuffer.len() - command.len();
            SnippetReplacement {
                start_index: command_start_index,
                end_index: command_start_index,
                snippet_prefix: "",
                snippet_suffix: " ",
            }
        }
    };

    Some(ExpandResult {
        lbuffer,
        rbuffer,
        replacement,
        left_snippet: matched.left_snippet(),
        right_snippet: matched.right_snippet(),
        evaluate: matched.evaluate(),
        has_placeholder: matched.has_placeholder(),
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
                abbr: 'null'
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

              - name: sudo apt install -y
                abbr: install
                snippet: sudo apt install -y
                action: replace-all
                global: true
                context: '^apt '

              - name: cd ..
                abbr: ..
                snippet: cd
                action: prepend
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
                    lbuffer: "g",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 1,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "git",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "simple abbr with rbuffer",
                lbuffer: "g",
                rbuffer: " --pager=never",
                expected: Some(ExpandResult {
                    lbuffer: "g",
                    rbuffer: " --pager=never",
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 1,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "git",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; g",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 12,
                        end_index: 13,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "git",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello null",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 11,
                        end_index: 15,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: ">/dev/null",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                rbuffer: " -m hello",
                expected: Some(ExpandResult {
                    lbuffer: "echo hello; git c",
                    rbuffer: " -m hello",
                    replacement: SnippetReplacement {
                        start_index: 16,
                        end_index: 17,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "commit",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
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
                    lbuffer: "home",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 4,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "$HOME",
                    right_snippet: "",
                    evaluate: true,
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "simple abbr with placeholder",
                lbuffer: "git cm",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "git cm",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 4,
                        end_index: 6,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "commit -m '",
                    right_snippet: "'",
                    evaluate: false,
                    has_placeholder: true,
                }),
            },
            Scenario {
                testname: "replace-all action",
                lbuffer: "apt install",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "apt install",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 11,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "sudo apt install -y",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "prepend action",
                lbuffer: "..",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "..",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 0,
                        snippet_prefix: "",
                        snippet_suffix: " ",
                    },
                    left_snippet: "cd",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "prepend action 2",
                lbuffer: "pwd; ..",
                rbuffer: "",
                expected: Some(ExpandResult {
                    lbuffer: "pwd; ..",
                    rbuffer: "",
                    replacement: SnippetReplacement {
                        start_index: 5,
                        end_index: 5,
                        snippet_prefix: "",
                        snippet_suffix: " ",
                    },
                    left_snippet: "cd",
                    right_snippet: "",
                    evaluate: false,
                    has_placeholder: false,
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
