use crate::config::abbrev::{Abbrev, Action, Match};
use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct ExpandResult<'a> {
    pub replacement: SnippetReplacement<'a>,
    pub left_snippet: &'a str,
    pub right_snippet: &'a str,
    pub condition: Option<&'a str>,
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
    let config = Config::load_or_exit();

    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let results = expand(&config, lbuffer);
    if results.is_empty() {
        return;
    }

    let mut has_if = false;
    for result in results {
        let snippet_start_index = result.replacement.start_index;
        let snippet_end_index = result.replacement.end_index;

        let lbuffer_pre = escape(Cow::from(&lbuffer[..snippet_start_index]));
        let lbuffer_post = escape(Cow::from(&lbuffer[snippet_end_index..]));
        let snippet_prefix = escape(Cow::from(result.replacement.snippet_prefix));
        let snippet_suffix = escape(Cow::from(result.replacement.snippet_suffix));
        let left_snippet = escape(Cow::from(result.left_snippet));
        let right_snippet = escape(Cow::from(result.right_snippet));
        let condition = result.condition.map(|c| escape(Cow::from(c)));

        let rbuffer = escape(Cow::from(rbuffer));
        let evaluate = if result.evaluate { "(e)" } else { "" };

        if let Some(condition) = &condition {
            if !has_if {
                print!(r#"if eval {condition};then "#);
            } else {
                print!(r#"elif eval {condition};then "#);
            }
            has_if = true;
        } else if has_if {
            print!(r"else ");
        }

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

        if condition.is_none() {
            break;
        }
    }

    if has_if {
        print!(r"fi");
    }

    println!();
}

fn expand<'a>(config: &'a Config, lbuffer: &'a str) -> Vec<ExpandResult<'a>> {
    let command = {
        let command_index = find_last_command_index(lbuffer);
        lbuffer[command_index..].trim_start()
    };

    let (_, last_arg) = command
        .rsplit_once(char::is_whitespace)
        .unwrap_or(("", command));

    let command_start_index = lbuffer.len() - command.len();
    let command_end_index = lbuffer.len();
    let last_arg_start_index = lbuffer.len() - last_arg.len();

    if last_arg.is_empty() {
        return Vec::new();
    }

    let matches = find_matches(&config.abbrevs, command, last_arg);

    matches
        .iter()
        .map(|m| ExpandResult {
            replacement: replacement_for(
                m.action(),
                command_start_index,
                command_end_index,
                last_arg_start_index,
            ),
            left_snippet: m.left_snippet(),
            right_snippet: m.right_snippet(),
            condition: m.condition(),
            evaluate: m.evaluate(),
            has_placeholder: m.has_placeholder(),
        })
        .collect()
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
                if: (( ${+commands[apt]} ))

              - name: trash
                abbr: rm
                snippet: trash
                if: (( ${+commands[trash]} ))

              - name: rm -r
                abbr: rm
                snippet: rm -r

              - name: never matched
                abbr: rm
                snippet: never

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
            pub expected: Vec<ExpandResult<'a>>,
        }

        let scenarios = &[
            Scenario {
                testname: "empty",
                lbuffer: "",
                expected: Vec::new(),
            },
            Scenario {
                testname: "simple abbr",
                lbuffer: "g",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 1,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "git",
                    right_snippet: "",
                    condition: None,
                    evaluate: false,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 12,
                        end_index: 13,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "git",
                    right_snippet: "",
                    condition: None,
                    evaluate: false,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 11,
                        end_index: 15,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: ">/dev/null",
                    right_snippet: "",
                    condition: None,
                    evaluate: false,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 16,
                        end_index: 17,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "commit",
                    right_snippet: "",
                    condition: None,
                    evaluate: false,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "global abbr with miss matched context",
                lbuffer: "echo git c",
                expected: Vec::new(),
            },
            Scenario {
                testname: "no matched abbr",
                lbuffer: "echo",
                expected: Vec::new(),
            },
            Scenario {
                testname: "simple abbr with evaluate=true",
                lbuffer: "home",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 4,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "$HOME",
                    right_snippet: "",
                    condition: None,
                    evaluate: true,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "simple abbr with placeholder",
                lbuffer: "git cm",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 4,
                        end_index: 6,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "commit -m '",
                    right_snippet: "'",
                    condition: None,
                    evaluate: false,
                    has_placeholder: true,
                }],
            },
            Scenario {
                testname: "replace-all action",
                lbuffer: "apt install",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 11,
                        snippet_prefix: "",
                        snippet_suffix: "",
                    },
                    left_snippet: "sudo apt install -y",
                    right_snippet: "",
                    condition: Some("(( ${+commands[apt]} ))"),
                    evaluate: false,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "prepend action",
                lbuffer: "..",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 0,
                        end_index: 0,
                        snippet_prefix: "",
                        snippet_suffix: " ",
                    },
                    left_snippet: "cd",
                    right_snippet: "",
                    condition: None,
                    evaluate: false,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "prepend action 2",
                lbuffer: "pwd; ..",
                expected: vec![ExpandResult {
                    replacement: SnippetReplacement {
                        start_index: 5,
                        end_index: 5,
                        snippet_prefix: "",
                        snippet_suffix: " ",
                    },
                    left_snippet: "cd",
                    right_snippet: "",
                    condition: None,
                    evaluate: false,
                    has_placeholder: false,
                }],
            },
            Scenario {
                testname: "conditional",
                lbuffer: "rm",
                expected: vec![
                    ExpandResult {
                        replacement: SnippetReplacement {
                            start_index: 0,
                            end_index: 2,
                            snippet_prefix: "",
                            snippet_suffix: "",
                        },
                        left_snippet: "trash",
                        right_snippet: "",
                        condition: Some("(( ${+commands[trash]} ))"),
                        evaluate: false,
                        has_placeholder: false,
                    },
                    ExpandResult {
                        replacement: SnippetReplacement {
                            start_index: 0,
                            end_index: 2,
                            snippet_prefix: "",
                            snippet_suffix: "",
                        },
                        left_snippet: "rm -r",
                        right_snippet: "",
                        condition: None,
                        evaluate: false,
                        has_placeholder: false,
                    },
                ],
            },
        ];

        for s in scenarios {
            let actual = expand(&config, s.lbuffer);

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

fn find_matches<'a>(abbrevs: &'a [Abbrev], command: &'a str, last_arg: &'a str) -> Vec<Match<'a>> {
    let mut matches = Vec::new();
    for abbr in abbrevs {
        if let Some(m) = abbr.do_match(command, last_arg) {
            let has_condition = m.condition().is_some();
            matches.push(m);

            if !has_condition {
                // Early break if m does not have condition.
                break;
            }
        }
    }
    matches
}

fn replacement_for(
    action: &Action,
    command_start_index: usize,
    command_end_index: usize,
    last_arg_start_index: usize,
) -> SnippetReplacement<'static> {
    match action {
        Action::ReplaceLast => SnippetReplacement {
            start_index: last_arg_start_index,
            end_index: command_end_index,
            snippet_prefix: "",
            snippet_suffix: "",
        },
        Action::ReplaceAll => SnippetReplacement {
            start_index: command_start_index,
            end_index: command_end_index,
            snippet_prefix: "",
            snippet_suffix: "",
        },
        Action::Prepend => SnippetReplacement {
            start_index: command_start_index,
            end_index: command_start_index,
            snippet_prefix: "",
            snippet_suffix: " ",
        },
    }
}
