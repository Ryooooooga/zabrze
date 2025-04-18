use ansi_term::Color;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExpandError {
    #[error("invalid regex: {0}")]
    RegexError(#[from] regex::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Trigger {
    #[serde(rename = "abbr")]
    Abbr(String),
    #[serde(rename = "abbr-pattern")]
    Regex(String),
}

impl Trigger {
    fn match_pattern(&self, last_arg: &str) -> Result<bool, ExpandError> {
        match self {
            Trigger::Abbr(abbr) => Ok(abbr == last_arg),
            Trigger::Regex(regex) => {
                let pattern = Regex::new(regex)?;
                Ok(pattern.is_match(last_arg))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "replace-last")]
    ReplaceLast,
    #[serde(rename = "replace-all")]
    ReplaceAll,
}

impl Default for Action {
    fn default() -> Self {
        Self::ReplaceLast
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Abbrev {
    pub name: Option<String>,

    #[serde(flatten)]
    pub trigger: Trigger,

    pub snippet: String,

    #[serde(default = "default_cursor")]
    pub cursor: Option<String>,

    #[serde(default)]
    pub action: Action,

    pub context: Option<String>,

    #[serde(rename = "if")]
    pub condition: Option<String>,

    #[serde(default = "default_as_false")]
    pub global: bool,

    #[serde(default = "default_as_false")]
    pub evaluate: bool,
}

impl Abbrev {
    pub fn do_match(&self, command: &str, last_arg: &str) -> Option<Match> {
        match self.do_match_impl(command, last_arg) {
            Ok(m) => m,
            Err(error) => {
                let name = self.name.as_ref().unwrap_or(&self.snippet);
                let error_message = format!("abbrev '{}': {}", name, error);
                let error_style = Color::Red.normal();

                eprintln!("{}", error_style.paint(error_message));
                None
            }
        }
    }

    fn do_match_impl(&self, command: &str, last_arg: &str) -> Result<Option<Match>, ExpandError> {
        if !(self.global || command == last_arg) {
            return Ok(None);
        }

        if !self.trigger.match_pattern(last_arg)? {
            return Ok(None);
        }

        if !self.match_context(command)? {
            return Ok(None);
        }

        let matched_snippet = self
            .cursor
            .as_ref()
            .and_then(|cursor| self.snippet.split_once(cursor))
            .map(|(left, right)| MatchedSnippet::WithPlaceholder { left, right })
            .unwrap_or_else(|| MatchedSnippet::Simple(&self.snippet));

        Ok(Some(Match {
            abbrev: self,
            matched_snippet,
        }))
    }

    fn match_context(&self, command: &str) -> Result<bool, ExpandError> {
        let context = match &self.context {
            Some(context) => context,
            None => return Ok(true), // No context means always match
        };

        let context_pattern = Regex::new(context)?;
        Ok(context_pattern.is_match(command))
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

    pub fn condition(&self) -> Option<&'a str> {
        self.abbrev.condition.as_deref()
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
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
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
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
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
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
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
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: Some("^echo ".to_string()),
                    condition: None,
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
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: Some("^printf ".to_string()),
                    condition: None,
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
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: Some("(echo".to_string()),
                    condition: None,
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
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TE{}ST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
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
            Scenario {
                testname: "should not match if cursor is none",
                abbr: Abbrev {
                    name: None,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TE{}ST".to_string(),
                    cursor: None,
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                },
                command: "test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TE{}ST",
                    right: "",
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should match with custom placeholder",
                abbr: Abbrev {
                    name: None,
                    trigger: Trigger::Abbr("test".to_string()),
                    snippet: "TE👇ST".to_string(),
                    cursor: Some("👇".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
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
            Scenario {
                testname: "should match abbrev-pattern",
                abbr: Abbrev {
                    name: None,
                    trigger: Trigger::Regex(r"\.py$".to_string()),
                    snippet: "python3".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                },
                command: "test.py",
                last_arg: "test.py",
                expected: Some(TestMatch {
                    left: "python3",
                    right: "",
                    has_placeholder: false,
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
                _ => panic!("{}", s.testname),
            }
        }
    }
}

fn default_cursor() -> Option<String> {
    Some("{}".to_string())
}

fn default_as_false() -> bool {
    false
}
