pub mod config_path;

pub use config_path::default_config_path;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Abbrev {
    pub name: Option<String>,
    pub abbr: String,
    pub snippet: String,
    pub context: Option<String>,
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
}
