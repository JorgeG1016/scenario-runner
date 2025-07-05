use serde::Deserialize;
use serde_json;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    commands_path: PathBuf,
    results_path: PathBuf,
}

pub fn parse_config_file(config_file: PathBuf) -> Config {}
