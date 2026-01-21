use ansi_term::Color;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExpandError {
    #[error("invalid regex: {0}")]
    Regex(#[from] regex::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Trigger {
    #[serde(rename = "trigger", alias = "abbr")]
    Text(String),
    #[serde(rename = "trigger-pattern", alias = "abbr-pattern")]
    Regex(String),
}

impl Trigger {
    fn match_pattern<'a>(
        &'a self,
        last_arg: &'a str,
    ) -> Result<Option<Vec<Capture<'a>>>, ExpandError> {
        match self {
            Trigger::Text(trigger) if trigger == last_arg => Ok(Some(vec![])),
            Trigger::Text(_) => Ok(None),
            Trigger::Regex(regex) => {
                let pattern = Regex::new(regex)?;

                let matches = pattern.captures(last_arg);
                let matches = match matches {
                    Some(matches) => matches,
                    None => return Ok(None),
                };

                let captures = pattern
                    .capture_names()
                    .flatten()
                    .filter_map(|name| {
                        matches.name(name).map(|value| Capture {
                            name: name.to_string(),
                            value: value.as_str(),
                        })
                    })
                    .collect();

                Ok(Some(captures))
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Capture<'a> {
    pub name: String,
    pub value: &'a str,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "replace-last")]
    #[default]
    ReplaceLast,
    #[serde(rename = "replace-all")]
    ReplaceAll,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snippet {
    pub name: Option<String>,

    #[serde(flatten)]
    pub trigger: Trigger,

    #[serde(rename = "snippet")]
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

    #[serde(rename = "abort-on-error", default = "default_as_false")]
    pub abort_on_error: bool,
}

impl Snippet {
    pub fn do_match<'a>(&'a self, command: &str, last_arg: &'a str) -> Option<Match<'a>> {
        match self.do_match_impl(command, last_arg) {
            Ok(m) => m,
            Err(error) => {
                let name = self.name.as_ref().unwrap_or(&self.snippet);
                let error_message = format!("snippet '{}': {}", name, error);
                let error_style = Color::Red.normal();

                eprintln!("{}", error_style.paint(error_message));
                None
            }
        }
    }

    fn do_match_impl<'a>(
        &'a self,
        command: &str,
        last_arg: &'a str,
    ) -> Result<Option<Match<'a>>, ExpandError> {
        if !(self.global || command == last_arg) {
            return Ok(None);
        }

        let captures = match self.trigger.match_pattern(last_arg)? {
            Some(captures) => captures,
            None => return Ok(None),
        };

        if !self.match_context(command)? {
            return Ok(None);
        }

        let matched_snippet = self
            .cursor
            .as_ref()
            .filter(|cursor| !cursor.is_empty())
            .and_then(|cursor| self.snippet.split_once(cursor))
            .map(|(left, right)| MatchedSnippet::WithPlaceholder { left, right })
            .unwrap_or_else(|| MatchedSnippet::Simple(&self.snippet));

        Ok(Some(Match {
            snippet: self,
            matched_snippet,
            captures,
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
    snippet: &'a Snippet,
    matched_snippet: MatchedSnippet<'a>,
    pub captures: Vec<Capture<'a>>,
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
        matches!(
            self.matched_snippet,
            MatchedSnippet::WithPlaceholder { left: _, right: _ }
        )
    }

    pub fn action(&self) -> &'a Action {
        &self.snippet.action
    }

    pub fn condition(&self) -> Option<&'a str> {
        self.snippet.condition.as_deref()
    }

    pub fn evaluate(&self) -> bool {
        self.snippet.evaluate
    }

    pub fn abort_on_error(&self) -> bool {
        self.snippet.abort_on_error
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
        struct TestCapture {
            name: &'static str,
            value: &'static str,
        }

        struct TestMatch {
            left: &'static str,
            right: &'static str,
            captures: &'static [TestCapture],
            has_placeholder: bool,
        }

        struct Scenario {
            testname: &'static str,
            snippet: Snippet,
            command: &'static str,
            last_arg: &'static str,
            expected: Option<TestMatch>,
        }

        let scenarios = &[
            Scenario {
                testname: "should match non-global if first arg",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TEST",
                    right: "",
                    captures: &[],
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should not match non-global if second arg",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: None,
            },
            Scenario {
                testname: "should match global",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: true,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TEST",
                    right: "",
                    captures: &[],
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should match global with context",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: Some("^echo ".to_string()),
                    condition: None,
                    global: true,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TEST",
                    right: "",
                    captures: &[],
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should not match global with context",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: Some("^printf ".to_string()),
                    condition: None,
                    global: true,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: None,
            },
            Scenario {
                testname: "should not match if context is invalid",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TEST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: Some("(echo".to_string()),
                    condition: None,
                    global: true,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "echo test",
                last_arg: "test",
                expected: None,
            },
            Scenario {
                testname: "should match with placeholder",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TE{}ST".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TE",
                    right: "ST",
                    captures: &[],
                    has_placeholder: true,
                }),
            },
            Scenario {
                testname: "should not match if cursor is none",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TE{}ST".to_string(),
                    cursor: None,
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TE{}ST",
                    right: "",
                    captures: &[],
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should match with custom placeholder",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Text("test".to_string()),
                    snippet: "TE👇ST".to_string(),
                    cursor: Some("👇".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "test",
                last_arg: "test",
                expected: Some(TestMatch {
                    left: "TE",
                    right: "ST",
                    captures: &[],
                    has_placeholder: true,
                }),
            },
            Scenario {
                testname: "should match trigger-pattern",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Regex(r"\.py$".to_string()),
                    snippet: "python3".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: "test.py",
                last_arg: "test.py",
                expected: Some(TestMatch {
                    left: "python3",
                    right: "",
                    captures: &[],
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should capture named groups (?P<...>)",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Regex(r"^\.(?P<digits>\d+)$".to_string()),
                    snippet: r".\$$n".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: ".3",
                last_arg: ".3",
                expected: Some(TestMatch {
                    left: r".\$$n",
                    right: "",
                    captures: &[TestCapture {
                        name: "digits",
                        value: "3",
                    }],
                    has_placeholder: false,
                }),
            },
            Scenario {
                testname: "should capture named groups (?<...>)",
                snippet: Snippet {
                    name: None,
                    trigger: Trigger::Regex(r"^\.(?<digits>\d+)$".to_string()),
                    snippet: r".\$$n".to_string(),
                    cursor: Some("{}".to_string()),
                    action: Action::ReplaceLast,
                    context: None,
                    condition: None,
                    global: false,
                    evaluate: false,
                    abort_on_error: false,
                },
                command: ".42",
                last_arg: ".42",
                expected: Some(TestMatch {
                    left: r".\$$n",
                    right: "",
                    captures: &[TestCapture {
                        name: "digits",
                        value: "42",
                    }],
                    has_placeholder: false,
                }),
            },
        ];

        for s in scenarios {
            let actual = s.snippet.do_match(s.command, s.last_arg);

            match (actual, &s.expected) {
                (Some(actual), Some(expected)) => {
                    assert_eq!(actual.left_snippet(), expected.left, "{}", s.testname);
                    assert_eq!(actual.right_snippet(), expected.right, "{}", s.testname);

                    assert_eq!(
                        actual.captures.len(),
                        expected.captures.len(),
                        "{}",
                        s.testname
                    );
                    for (i, capture) in actual.captures.iter().enumerate() {
                        assert_eq!(
                            capture.name, expected.captures[i].name,
                            "{}.captures[{i}]",
                            s.testname
                        );
                        assert_eq!(
                            capture.value, expected.captures[i].value,
                            "{}.captures[{i}]",
                            s.testname
                        );
                    }

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
