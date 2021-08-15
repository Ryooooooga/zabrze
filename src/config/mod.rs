pub mod abbrev;
pub mod config_path;

pub use abbrev::Abbrev;
pub use config_path::default_config_path;

use ansi_term::Color;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub abbrevs: Vec<Abbrev>,
}

impl Config {
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
        let path = &default_config_path().expect("could not determine config file path");

        let config = Self::load_from_file(path).unwrap_or_else(|err| {
            let path = path.to_string_lossy();
            let error_message = format!("failed to load config `{}': {}", path, err);
            let error_style = Color::Red.normal();

            eprintln!("{}", error_style.paint(error_message));
            std::process::exit(1);
        });

        config
    }
}
