use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct ExpandResult {
    pub buffer: String,
    pub cursor: usize,
}

pub fn run(args: &ExpandArgs) {
    if let Some(result) = expand(args, &Config::load_or_exit()) {
        println!(
            r"BUFFER={};CURSOR={}",
            escape(Cow::from(result.buffer)),
            result.cursor
        );
    }
}

fn expand(args: &ExpandArgs, config: &Config) -> Option<ExpandResult> {
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

    let new_lbuffer = format!("{}{}", lbuffer_without_last_arg, abbrev.snippet);
    let buffer = format!("{}{}", new_lbuffer, rbuffer);
    let cursor = new_lbuffer.chars().count();

    Some(ExpandResult { buffer, cursor })
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
            ",
        )
        .unwrap()
    }

    #[test]
    fn test_expand() {
        let config = test_config();

        struct Scenario {
            pub testname: &'static str,
            pub lbuffer: &'static str,
            pub rbuffer: &'static str,
            pub expected: Option<ExpandResult>,
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
                    buffer: "git".to_string(),
                    cursor: 3,
                }),
            },
            Scenario {
                testname: "simple abbr with rbuffer",
                lbuffer: "g",
                rbuffer: " --pager=never",
                expected: Some(ExpandResult {
                    buffer: "git --pager=never".to_string(),
                    cursor: 3,
                }),
            },
            Scenario {
                testname: "simple abbr with leading command",
                lbuffer: "echo hello; g",
                rbuffer: "",
                expected: Some(ExpandResult {
                    buffer: "echo hello; git".to_string(),
                    cursor: 15,
                }),
            },
            Scenario {
                testname: "global abbr",
                lbuffer: "echo hello null",
                rbuffer: "",
                expected: Some(ExpandResult {
                    buffer: "echo hello >/dev/null".to_string(),
                    cursor: 21,
                }),
            },
            Scenario {
                testname: "global abbr with context",
                lbuffer: "echo hello; git c",
                rbuffer: " -m hello",
                expected: Some(ExpandResult {
                    buffer: "echo hello; git commit -m hello".to_string(),
                    cursor: 22,
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
        ];

        for s in scenarios {
            let actual = expand(
                &ExpandArgs {
                    lbuffer: s.lbuffer.to_string(),
                    rbuffer: s.rbuffer.to_string(),
                },
                &config,
            );

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
