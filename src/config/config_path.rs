use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

static ZABRZE_CONFIG_FILE_ENV_KEY: &str = "ZABRZE_CONFIG_FILE";
static XDG_CONFIG_HOME_ENV_KEY: &str = "XDG_CONFIG_HOME";

static DEFAULT_CONFIG_DIR: &str = "zabrze";
static DEFAULT_CONFIG_FILE: &str = "config.yaml";

trait ConfigPath {
    fn env(&self, key: &str) -> Option<OsString>;
    fn home(&self) -> Option<PathBuf>;
}

#[derive(Debug)]
struct ConfigPathImpl {}

impl ConfigPath for ConfigPathImpl {
    fn env(&self, key: &str) -> Option<OsString> {
        env::var_os(key)
    }
    fn home(&self) -> Option<PathBuf> {
        dirs::home_dir()
    }
}

fn get_default_path<C: ConfigPath>(c: &C) -> Option<PathBuf> {
    // Return $ZABRZE_CONFIG_FILE if defined
    if let Some(zabrze_config_file) = c.env(ZABRZE_CONFIG_FILE_ENV_KEY).map(PathBuf::from) {
        return Some(zabrze_config_file);
    }

    // Get ${XDG_CONFIG_HOME:-$HOME/.config}
    let config_home =
        if let Some(xdg_config_home) = c.env(XDG_CONFIG_HOME_ENV_KEY).map(PathBuf::from) {
            xdg_config_home
        } else {
            let mut path = c.home()?;
            path.push(".config");
            path
        };

    // Return $config_path/zabrze/config.yaml
    let mut config_path = config_home;
    config_path.push(DEFAULT_CONFIG_DIR);
    config_path.push(DEFAULT_CONFIG_FILE);
    Some(config_path)
}

pub fn default_config_path() -> Option<PathBuf> {
    get_default_path(&ConfigPathImpl {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct DummyConfigPath {
        envs: HashMap<&'static str, &'static str>,
        home: &'static str,
    }

    impl ConfigPath for DummyConfigPath {
        fn env(&self, key: &str) -> Option<OsString> {
            self.envs.get(key).map(OsString::from)
        }
        fn home(&self) -> Option<PathBuf> {
            Some(PathBuf::from(self.home))
        }
    }

    #[test]
    fn test_config_path() {
        struct Scenario {
            pub testname: &'static str,
            pub envs: HashMap<&'static str, &'static str>,
            pub home: &'static str,
            pub expected: &'static str,
        }

        let scenarios = [
            Scenario {
                testname: "follow ZABRZE_CONFIG_FILE",
                envs: vec![
                    ("ZABRZE_CONFIG_FILE", "/home/user/.zabrze.yaml"),
                    ("XDG_CONFIG_HOME", "/home/user/.xdgConfig"),
                ]
                .into_iter()
                .collect(),
                home: "/home/user/",
                expected: "/home/user/.zabrze.yaml",
            },
            Scenario {
                testname: "follow XDG_CONFIG_HOME",
                envs: vec![("XDG_CONFIG_HOME", "/home/user/.xdgConfig")]
                    .into_iter()
                    .collect(),
                home: "/home/user/",
                expected: "/home/user/.xdgConfig/zabrze/config.yaml",
            },
            Scenario {
                testname: "use default path",
                envs: HashMap::new(),
                home: "/home/user/",
                expected: "/home/user/.config/zabrze/config.yaml",
            },
        ];

        for s in &scenarios {
            let c = DummyConfigPath {
                envs: s.envs.clone(),
                home: s.home,
            };

            let expected = Some(PathBuf::from(s.expected));

            assert_eq!(get_default_path(&c), expected, "{}", s.testname);
        }
    }
}
