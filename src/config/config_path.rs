use std::env;
use std::ffi::OsString;

static ZABRZE_CONFIG_HOME_ENV_KEY: &str = "ZABRZE_CONFIG_HOME";
static XDG_CONFIG_HOME_ENV_KEY: &str = "XDG_CONFIG_HOME";
static HOME_ENV_KEY: &str = "HOME";

static DEFAULT_CONFIG_DIR: &str = "zabrze";

trait ConfigPath {
    fn env(&self, key: &str) -> Option<OsString>;
}

#[derive(Debug)]
struct ConfigPathImpl {}

impl ConfigPath for ConfigPathImpl {
    fn env(&self, key: &str) -> Option<OsString> {
        env::var_os(key)
    }
}

fn get_default_dir<C: ConfigPath>(c: &C) -> Option<String> {
    // Return $ZABRZE_CONFIG_HOME if defined
    if let Some(zabrze_config_home) = c.env(ZABRZE_CONFIG_HOME_ENV_KEY) {
        return zabrze_config_home.to_str().map(String::from);
    }

    // Get ${XDG_CONFIG_HOME:-$HOME/.config}
    if let Some(xdg_config_home) = c.env(XDG_CONFIG_HOME_ENV_KEY) {
        return xdg_config_home
            .to_str()
            .map(|xdg_config_home| format!("{xdg_config_home}/{DEFAULT_CONFIG_DIR}"));
    }

    let home = c.env(HOME_ENV_KEY)?;
    home.to_str()
        .map(|home| format!("{home}/.config/{DEFAULT_CONFIG_DIR}"))
}

pub fn get_default_config_dir() -> Option<String> {
    get_default_dir(&ConfigPathImpl {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct DummyConfigPath {
        envs: HashMap<&'static str, &'static str>,
    }

    impl ConfigPath for DummyConfigPath {
        fn env(&self, key: &str) -> Option<OsString> {
            self.envs.get(key).map(OsString::from)
        }
    }

    #[test]
    fn test_config_path() {
        struct Scenario {
            pub testname: &'static str,
            pub envs: HashMap<&'static str, &'static str>,
            pub expected: &'static str,
        }

        let scenarios = [
            Scenario {
                testname: "follow ZABRZE_CONFIG_HOME",
                envs: vec![
                    ("ZABRZE_CONFIG_HOME", "/home/user/.zabrze"),
                    ("XDG_CONFIG_HOME", "/home/user/.xdgConfig"),
                    ("HOME", "/home/user"),
                ]
                .into_iter()
                .collect(),
                expected: "/home/user/.zabrze",
            },
            Scenario {
                testname: "follow XDG_CONFIG_HOME",
                envs: vec![
                    ("XDG_CONFIG_HOME", "/home/user/.xdgConfig"),
                    ("HOME", "/home/user"),
                ]
                .into_iter()
                .collect(),
                expected: "/home/user/.xdgConfig/zabrze",
            },
            Scenario {
                testname: "use default path",
                envs: vec![("HOME", "/home/user")].into_iter().collect(),
                expected: "/home/user/.config/zabrze",
            },
        ];

        for s in &scenarios {
            let c = DummyConfigPath {
                envs: s.envs.clone(),
            };

            assert_eq!(
                get_default_dir(&c),
                Some(s.expected.to_string()),
                "{}",
                s.testname
            );
        }
    }
}
