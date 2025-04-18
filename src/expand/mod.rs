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
pub struct ExpansionVariable<'a> {
    pub name: String,
    pub value: &'a str,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Expansion<'a> {
    pub replacing_index: usize,
    pub left_snippet: &'a str,
    pub right_snippet: &'a str,
    pub condition: Option<&'a str>,
    pub variables: Vec<ExpansionVariable<'a>>,
    pub evaluate: bool,
    pub has_placeholder: bool,
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
        let prefix = escape(Cow::from(&lbuffer[..expansion.replacing_index]));
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

        for var in expansion.variables.iter() {
            let name = escape(Cow::from(&var.name));
            let value = escape(Cow::from(var.value));
            print!(r#"local {name}={value};"#);
        }

        print!(r"local left_snippet={left_snippet};");
        if expansion.has_placeholder {
            print!(r"local right_snippet={right_snippet};");
            print!(r#"LBUFFER={prefix}"${{{evaluate}left_snippet}}";"#);
            print!(r#"RBUFFER="${{{evaluate}right_snippet}}"{rbuffer};"#);
            print!(r"__zabrze_has_placeholder=1;");
        } else {
            print!(r#"LBUFFER={prefix}"${{{evaluate}left_snippet}}";"#);
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

    if last_arg.is_empty() {
        return ExpandResult {
            command,
            last_arg,
            expansions: Vec::new(),
        };
    }

    let last_arg_start_index = lbuffer.len() - last_arg.len();
    let command_start_index = lbuffer.len() - command.len();

    let matches = find_matches(&config.abbrevs, command, last_arg);

    let expansions = matches
        .iter()
        .map(|m| Expansion {
            replacing_index: match m.action() {
                Action::ReplaceLast => last_arg_start_index,
                Action::ReplaceAll => command_start_index,
            },
            left_snippet: m.left_snippet(),
            right_snippet: m.right_snippet(),
            condition: m.condition(),
            variables: m
                .captures
                .iter()
                .map(|c| ExpansionVariable {
                    name: c.name.to_string(),
                    value: c.value,
                })
                .collect(),
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

              - name: .N
                abbr-pattern: ^\.(?<n>\d+)$
                snippet: awk '{print \$$n}'
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
                        replacing_index: 0,
                        left_snippet: "git",
                        right_snippet: "",
                        condition: None,
                        variables: vec![],
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
                        replacing_index: 12,
                        left_snippet: "git",
                        right_snippet: "",
                        condition: None,
                        variables: vec![],
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
                        replacing_index: 11,
                        left_snippet: ">/dev/null",
                        right_snippet: "",
                        condition: None,
                        variables: vec![],
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
                        replacing_index: 16,
                        left_snippet: "commit",
                        right_snippet: "",
                        condition: None,
                        variables: vec![],
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
                        replacing_index: 0,
                        left_snippet: "$HOME",
                        right_snippet: "",
                        condition: None,
                        variables: vec![],
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
                        replacing_index: 4,
                        left_snippet: "commit -m '",
                        right_snippet: "'",
                        condition: None,
                        variables: vec![],
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
                        replacing_index: 0,
                        left_snippet: "sudo apt install -y",
                        right_snippet: "",
                        condition: Some("(( ${+commands[apt]} ))"),
                        variables: vec![],
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
                        replacing_index: 0,
                        left_snippet: "cd $abbr",
                        right_snippet: "",
                        condition: None,
                        variables: vec![],
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
                        replacing_index: 5,
                        left_snippet: "cd $abbr",
                        right_snippet: "",
                        condition: None,
                        variables: vec![],
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
                            replacing_index: 0,
                            left_snippet: "trash",
                            right_snippet: "",
                            condition: Some("(( ${+commands[trash]} ))"),
                            variables: vec![],
                            evaluate: false,
                            has_placeholder: false,
                        },
                        Expansion {
                            replacing_index: 0,
                            left_snippet: "rm -r",
                            right_snippet: "",
                            condition: None,
                            variables: vec![],
                            evaluate: false,
                            has_placeholder: false,
                        },
                    ],
                },
            },
            Scenario {
                testname: "with captures",
                lbuffer: ".2",
                expected: ExpandResult {
                    command: ".2",
                    last_arg: ".2",
                    expansions: vec![Expansion {
                        replacing_index: 0,
                        left_snippet: r"awk '{print \$$n}'",
                        right_snippet: "",
                        condition: None,
                        variables: vec![ExpansionVariable {
                            name: "n".to_string(),
                            value: "2",
                        }],
                        evaluate: true,
                        has_placeholder: false,
                    }],
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
