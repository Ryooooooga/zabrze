use ansi_term::Color;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "replace-last")]
    ReplaceLast,
    #[serde(rename = "replace-all")]
    ReplaceAll,
    #[serde(rename = "prepend")]
    Prepend,
}

impl Default for Action {
    fn default() -> Self {
        Self::ReplaceLast
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Abbrev {
    pub name: Option<String>,
    pub abbr: String,
    pub snippet: String,

    #[serde(default)]
    pub action: Action,

    pub context: Option<String>,

    #[serde(default = "default_as_false")]
    pub global: bool,

    #[serde(default = "default_as_false")]
    pub evaluate: bool,
}

impl Abbrev {
    pub fn do_match(&self, command: &str, last_arg: &str) -> Option<Match> {
        if self.abbr != last_arg {
            return None;
        }
        if !(self.global || command == last_arg) {
            return None;
        }

        // Check context
        let context_opt = match self.context.as_ref().map(|ctx| Regex::new(ctx)) {
            Some(Ok(context)) => Some(context),
            Some(Err(error)) => {
                let name = self.name.as_ref().unwrap_or(&self.snippet);
                let error_message = format!("invalid regex in abbrev '{}': {}", name, error);
                let error_style = Color::Red.normal();

                eprintln!("{}", error_style.paint(error_message));
                return None;
            }
            None => None,
        };

        let is_context_matched = context_opt
            .map(|context| context.is_match(command))
            .unwrap_or(true);

        if !is_context_matched {
            return None;
        }

        const PLACEHOLDER: &str = "{}";

        let matched_snippet = self
            .snippet
            .split_once(PLACEHOLDER)
            .map(|(left, right)| MatchedSnippet::WithPlaceholder { left, right })
            .unwrap_or_else(|| MatchedSnippet::Simple(&self.snippet));

        Some(Match {
            abbrev: self,
            matched_snippet,
        })
    }
}

#[derive(Debug)]
pub struct Match<'a> {
    abbrev: &'a Abbrev,
    matched_snippet: MatchedSnippet<'a>,
}

impl<'a> Match<'a> {
    pub fn left_snippet(&self) -> &'a str {
        match self.matched_snippet {
            MatchedSnippet::Simple(s) => s,
            MatchedSnippet::WithPlaceholder { left, right: _ } => left,
        }
    }

    pub fn right_snippet(&self) -> &'a str {
        match self.matched_snippet {
            MatchedSnippet::Simple(_) => "",
            MatchedSnippet::WithPlaceholder { left: _, right } => right,
        }
    }

    pub fn has_placeholder(&self) -> bool {
        match self.matched_snippet {
            MatchedSnippet::Simple(_) => false,
            MatchedSnippet::WithPlaceholder { left: _, right: _ } => true,
        }
    }

    pub fn action(&self) -> &'a Action {
        &self.abbrev.action
    }

    pub fn evaluate(&self) -> bool {
        self.abbrev.evaluate
    }
}

#[derive(Debug)]
pub enum MatchedSnippet<'a> {
    Simple(&'a str),
    WithPlaceholder { left: &'a str, right: &'a str },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_do_match() {
        pub struct TestMatch {
            left: &'static str,
            right: &'static str,
            has_placeholder: bool,
        }

        struct Scenario {
            testname: &'static str,
            abbr: Abbrev,
            command: &'static str,
            last_arg: &'static str,
            expected: Option<TestMatch>,
        }

        let scenarios = &[
            Scenario {
                testname: "should match non-global if first arg",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: "TEST".to_string(),
                    action: Action::ReplaceLast,
                    context: None,
                    global: false,
                    evaluate: false,
                },
                command: "test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TEST",
                    right: "",
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should not match non-global if second arg",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: "TEST".to_string(),
                    action: Action::ReplaceLast,
                    context: None,
                    global: false,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: None,
            },
            Scenario {
                testname: "should match global",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: "TEST".to_string(),
                    action: Action::ReplaceLast,
                    context: None,
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TEST",
                    right: "",
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should match global with context",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: "TEST".to_string(),
                    action: Action::ReplaceLast,
                    context: Some("^echo ".to_string()),
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TEST",
                    right: "",
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should not match global with context",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: "TEST".to_string(),
                    action: Action::ReplaceLast,
                    context: Some("^printf ".to_string()),
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: None,
            },
            Scenario {
                testname: "should not match if context is invalid",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: "TEST".to_string(),
                    action: Action::ReplaceLast,
                    context: Some("(echo".to_string()),
                    global: true,
                    evaluate: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: None,
            },
            Scenario {
                testname: "should match with placeholder",
                abbr: Abbrev {
                    name: None,
                    abbr: "test".to_string(),
                    snippet: "TE{}ST".to_string(),
                    action: Action::ReplaceLast,
                    context: None,
                    global: false,
                    evaluate: false,
                },
                command: "test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TE",
                    right: "ST",
                    has_placeholder: true,
                }),
            },
        ];

        for s in scenarios {
            let actual = s.abbr.do_match(s.command, s.last_arg);

            match (actual, &s.expected) {
                (Some(actual), Some(expected)) => {
                    assert_eq!(actual.left_snippet(), expected.left, "{}", s.testname);
                    assert_eq!(actual.right_snippet(), expected.right, "{}", s.testname);
                    assert_eq!(
                        actual.has_placeholder(),
                        expected.has_placeholder,
                        "{}",
                        s.testname
                    );
                }
                (None, None) => { /* ok */ }
                _ => assert!(false, "{}", s.testname),
            }
        }
    }
}

fn default_as_false() -> bool {
    false
}
