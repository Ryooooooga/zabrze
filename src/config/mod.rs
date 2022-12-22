pub mod abbrev;
pub mod config_path;

pub use abbrev::{Abbrev, Action, Trigger};
pub use config_path::get_default_config_dir;

use ansi_term::Color;
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub abbrevs: Vec<Abbrev>,
}

impl Config {
    #[allow(dead_code)]
    pub fn load_from_str(s: &str) -> Result<Self, ConfigError> {
        let config = serde_yaml::from_str(s)?;
        Ok(config)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let file = File::open(path)?;
        let config = serde_yaml::from_reader(&file)?;
        Ok(config)
    }

    pub fn load_or_exit() -> Self {
        let config_dir = get_default_config_dir().expect("could not determine config directory");
        let mut config: Config = Default::default();

        for path in &Self::config_file_paths(&config_dir) {
            match Self::load_from_file(path) {
                Ok(c) => config.merge(c),
                Err(err) => {
                    let error_message =
                        format!("failed to load config `{}': {}", path.display(), err);
                    let error_style = Color::Red.normal();

                    eprintln!("{}", error_style.paint(error_message));
                }
            };
        }

        config
    }

    fn merge(&mut self, mut other: Self) {
        self.abbrevs.append(&mut other.abbrevs);
    }

    fn config_file_paths(config_dir: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for ext in &["yaml", "yml"] {
            let pattern = format!("{config_dir}/**/*{ext}");

            for path in glob(&pattern).expect("failed to read glob pattern") {
                match path {
                    Ok(path) => paths.push(path),
                    Err(err) => {
                        let error_message = format!("failed to load config: {err}");
                        let error_style = Color::Red.normal();
                        eprintln!("{}", error_style.paint(error_message));
                    }
                }
            }
        }

        paths.sort();
        paths
    }
}
