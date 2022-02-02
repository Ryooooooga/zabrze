use ansi_term::Color;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Abbrev {
    pub name: Option<String>,
    pub abbr: String,
    pub snippet: String,
    pub context: Option<String>,

    #[serde(default = "default_as_false")]
    pub global: bool,

    #[serde(default = "default_as_false")]
    pub evaluate: bool,
}

impl Abbrev {
    pub fn is_match(&self, command: &str, last_arg: &str) -> bool {
        if self.abbr != last_arg {
            return false;
        }
        if !(self.global || command == last_arg) {
            return false;
        }

        let context_opt = match self.context.as_ref().map(|ctx| Regex::new(ctx)) {
            Some(Ok(context)) => Some(context),
            Some(Err(error)) => {
                let name = self.name.as_ref().unwrap_or(&self.snippet);
                let error_message = format!("invalid regex in abbrev '{}': {}", name, error);
                let error_style = Color::Red.normal();

                eprintln!("{}", error_style.paint(error_message));
                return false;
            }
            None => None,
        };

        context_opt
            .map(|context| context.is_match(command))
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_match() {
        struct Scenario {
            pub testname: &'static str,
            pub abbr: Abbrev,
            pub command: &'static str,
            pub last_arg: &'static str,
            pub expected: bool,
        }

        let scenarios = &[
            Scenario {
                testname: "should match non-global if first arg",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: String::new(),
                    context: None,
                    global: false,
                    evaluate: false,
                },
                command: "test",
                last_arg: "test",
                expected: true,
            },
            Scenario {
                testname: "should not match non-global if second arg",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: String::new(),
                    context: None,
                    global: false,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: false,
            },
            Scenario {
                testname: "should match global",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: String::new(),
                    context: None,
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: true,
            },
            Scenario {
                testname: "should match global with context",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: String::new(),
                    context: Some("^echo ".to_string()),
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: true,
            },
            Scenario {
                testname: "should not match global with context",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: String::new(),
                    context: Some("^printf ".to_string()),
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: false,
            },
            Scenario {
                testname: "should not match if context is invalid",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: String::new(),
                    context: Some("(echo".to_string()),
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: false,
            },
        ];

        for s in scenarios {
            assert_eq!(
                s.abbr.is_match(s.command, s.last_arg),
                s.expected,
                "{}",
                s.testname
            );
        }
    }
}

fn default_as_false() -> bool {
    false
}
