use serde::Deserialize;
use serde_json;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigErrors {
    #[error("Config File does not exist")]
    ConfigFileDoesNotExist,
    #[error("Config File could not be opened")]
    ConfigFileNotOpened(#[from] io::Error),
    #[error("Config File could not be parsed properly")]
    ConfigFileParsingFailed(#[from] serde_json::Error),
}

#[derive(Deserialize, Debug)]
pub struct Config {
    commands_path: PathBuf,
    results_path: PathBuf,
}

impl Config {
    pub fn new(config_file_path: PathBuf) -> Result<Self, ConfigErrors> {
        if !config_file_path.exists() {
            return Err(ConfigErrors::ConfigFileDoesNotExist);
        }

        let config_file = File::open(config_file_path)?;
        let config_reader = io::BufReader::new(config_file);
        let parsed_config: Config = serde_json::from_reader(config_reader)?;

        Ok(parsed_config)
    }
}

mod tests {}
