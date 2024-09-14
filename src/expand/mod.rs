use crate::config::abbrev::{Abbrev, Action, Match};
use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq)]
pub struct ExpandResult<'a> {
    pub command: &'a str,
    pub last_arg: &'a str,
    pub expansions: Vec<Expansion<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Expansion<'a> {
    pub name: &'a str,
    pub replacement: SnippetReplacement,
    pub left_snippet: &'a str,
    pub right_snippet: &'a str,
    pub condition: Option<&'a str>,
    pub evaluate: bool,
    pub has_placeholder: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SnippetReplacement {
    pub start_index: usize,
    pub end_index: usize,
}

pub fn run(args: &ExpandArgs) {
    let config = Config::load_or_exit();

    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let result = expand(&config, lbuffer);
    if result.expansions.is_empty() {
        return;
    }

    let command = escape(Cow::from(result.command));
    let abbr = escape(Cow::from(result.last_arg));

    print!(r#"local command={command};"#);
    print!(r#"local abbr={abbr};"#);

    let mut has_if = false;
    for expansion in &result.expansions {
        let snippet_start_index = expansion.replacement.start_index;
        let snippet_end_index = expansion.replacement.end_index;

        let name = escape(Cow::from(expansion.name));
        let lbuffer_pre = escape(Cow::from(&lbuffer[..snippet_start_index]));
        let lbuffer_post = escape(Cow::from(&lbuffer[snippet_end_index..]));
        let left_snippet = escape(Cow::from(expansion.left_snippet));
        let right_snippet = escape(Cow::from(expansion.right_snippet));
        let condition = expansion.condition.map(|c| escape(Cow::from(c)));

        let rbuffer = escape(Cow::from(rbuffer));
        let evaluate = if expansion.evaluate { "(e)" } else { "" };

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

        print!(r"local name={name};");
        print!(r"local left_snippet={left_snippet};");
        if expansion.has_placeholder {
            print!(r"local right_snippet={right_snippet};");
            print!(r#"LBUFFER={lbuffer_pre}"${{{evaluate}left_snippet}}";"#);
            print!(r#"RBUFFER="${{{evaluate}right_snippet}}"{lbuffer_post}{rbuffer};"#);
            print!(r"__zabrze_has_placeholder=1;");
        } else {
            print!(r#"LBUFFER={lbuffer_pre}"${{{evaluate}left_snippet}}"{lbuffer_post};"#);
            print!(r#"RBUFFER={rbuffer};"#);
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

fn expand<'a>(config: &'a Config, lbuffer: &'a str) -> ExpandResult<'a> {
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
        return ExpandResult {
            command,
            last_arg,
            expansions: Vec::new(),
        };
    }

    let matches = find_matches(&config.abbrevs, command, last_arg);

    let expansions = matches
        .iter()
        .map(|m| Expansion {
            name: m.name(),
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
        .collect();

    ExpandResult {
        command,
        last_arg,
        expansions,
    }
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
                abbr-pattern: \.\.$
                snippet: cd $abbr
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
            pub expected: ExpandResult<'a>,
        }

        let scenarios = &[
            Scenario {
                testname: "empty",
                lbuffer: "",
                expected: ExpandResult {
                    command: "",
                    last_arg: "",
                    expansions: Vec::new(),
                },
            },
            Scenario {
                testname: "simple abbr",
                lbuffer: "g",
                expected: ExpandResult {
                    command: "g",
                    last_arg: "g",
                    expansions: vec![Expansion {
                        name: "git",
                        replacement: SnippetReplacement {
                            start_index: 0,
                            end_index: 1,
                        },
                        left_snippet: "git",
                        right_snippet: "",
                        condition: None,
                        evaluate: false,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                expected: ExpandResult {
                    command: "g",
                    last_arg: "g",
                    expansions: vec![Expansion {
                        name: "git",
                        replacement: SnippetReplacement {
                            start_index: 12,
                            end_index: 13,
                        },
                        left_snippet: "git",
                        right_snippet: "",
                        condition: None,
                        evaluate: false,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                expected: ExpandResult {
                    command: "echo hello null",
                    last_arg: "null",
                    expansions: vec![Expansion {
                        name: ">/dev/null",
                        replacement: SnippetReplacement {
                            start_index: 11,
                            end_index: 15,
                        },
                        left_snippet: ">/dev/null",
                        right_snippet: "",
                        condition: None,
                        evaluate: false,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                expected: ExpandResult {
                    command: "git c",
                    last_arg: "c",
                    expansions: vec![Expansion {
                        name: "git commit",
                        replacement: SnippetReplacement {
                            start_index: 16,
                            end_index: 17,
                        },
                        left_snippet: "commit",
                        right_snippet: "",
                        condition: None,
                        evaluate: false,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "global abbr with miss matched context",
                lbuffer: "echo git c",
                expected: ExpandResult {
                    command: "echo git c",
                    last_arg: "c",
                    expansions: Vec::new(),
                },
            },
            Scenario {
                testname: "no matched abbr",
                lbuffer: "echo",
                expected: ExpandResult {
                    command: "echo",
                    last_arg: "echo",
                    expansions: Vec::new(),
                },
            },
            Scenario {
                testname: "simple abbr with evaluate=true",
                lbuffer: "home",
                expected: ExpandResult {
                    command: "home",
                    last_arg: "home",
                    expansions: vec![Expansion {
                        name: "$HOME",
                        replacement: SnippetReplacement {
                            start_index: 0,
                            end_index: 4,
                        },
                        left_snippet: "$HOME",
                        right_snippet: "",
                        condition: None,
                        evaluate: true,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "simple abbr with placeholder",
                lbuffer: "git cm",
                expected: ExpandResult {
                    command: "git cm",
                    last_arg: "cm",
                    expansions: vec![Expansion {
                        name: "git commit -m ''",
                        replacement: SnippetReplacement {
                            start_index: 4,
                            end_index: 6,
                        },
                        left_snippet: "commit -m '",
                        right_snippet: "'",
                        condition: None,
                        evaluate: false,
                        has_placeholder: true,
                    }],
                },
            },
            Scenario {
                testname: "replace-all action",
                lbuffer: "apt install",
                expected: ExpandResult {
                    command: "apt install",
                    last_arg: "install",
                    expansions: vec![Expansion {
                        name: "sudo apt install -y",
                        replacement: SnippetReplacement {
                            start_index: 0,
                            end_index: 11,
                        },
                        left_snippet: "sudo apt install -y",
                        right_snippet: "",
                        condition: Some("(( ${+commands[apt]} ))"),
                        evaluate: false,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "prepend action",
                lbuffer: "..",
                expected: ExpandResult {
                    command: "..",
                    last_arg: "..",
                    expansions: vec![Expansion {
                        name: "cd ..",
                        replacement: SnippetReplacement {
                            start_index: 0,
                            end_index: 2,
                        },
                        left_snippet: "cd $abbr",
                        right_snippet: "",
                        condition: None,
                        evaluate: true,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "prepend action 2",
                lbuffer: "pwd; ../..",
                expected: ExpandResult {
                    command: "../..",
                    last_arg: "../..",
                    expansions: vec![Expansion {
                        name: "cd ..",
                        replacement: SnippetReplacement {
                            start_index: 5,
                            end_index: 10,
                        },
                        left_snippet: "cd $abbr",
                        right_snippet: "",
                        condition: None,
                        evaluate: true,
                        has_placeholder: false,
                    }],
                },
            },
            Scenario {
                testname: "conditional",
                lbuffer: "rm",
                expected: ExpandResult {
                    command: "rm",
                    last_arg: "rm",
                    expansions: vec![
                        Expansion {
                            name: "trash",
                            replacement: SnippetReplacement {
                                start_index: 0,
                                end_index: 2,
                            },
                            left_snippet: "trash",
                            right_snippet: "",
                            condition: Some("(( ${+commands[trash]} ))"),
                            evaluate: false,
                            has_placeholder: false,
                        },
                        Expansion {
                            name: "rm -r",
                            replacement: SnippetReplacement {
                                start_index: 0,
                                end_index: 2,
                            },
                            left_snippet: "rm -r",
                            right_snippet: "",
                            condition: None,
                            evaluate: false,
                            has_placeholder: false,
                        },
                    ],
                },
            },
        ];

        for s in scenarios {
            let actual = expand(&config, s.lbuffer);

            assert_eq!(actual, s.expected, "{}", s.testname);
        }
    }
}

fn find_last_command_index(line: &str) -> usize {
    line.rfind([';', '&', '|', '(', '`', '\n'])
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
) -> SnippetReplacement {
    match action {
        Action::ReplaceLast => SnippetReplacement {
            start_index: last_arg_start_index,
            end_index: command_end_index,
        },
        Action::ReplaceAll => SnippetReplacement {
            start_index: command_start_index,
            end_index: command_end_index,
        },
    }
}
