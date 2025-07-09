use serde::Deserialize;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigErrors {
    #[error("Config File does not exist")]
    DoesNotExist,
    #[error("Config File could not be opened")]
    NotOpened(#[from] io::Error),
    #[error("Config File could not be parsed properly")]
    ParsingFailed(#[from] serde_json::Error),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    commands_path: PathBuf,
    results_path: PathBuf,
}

impl Config {
    pub fn new(config_file_path: PathBuf) -> Result<Self, ConfigErrors> {
        if !config_file_path.exists() {
            return Err(ConfigErrors::DoesNotExist);
        }

        let config_file = File::open(config_file_path)?;
        let config_reader = io::BufReader::new(config_file);
        let parsed_config: Config = serde_json::from_reader(config_reader)?;

        Ok(parsed_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::NamedTempFile;

    use std::io::Write;

    #[test]
    fn config_new_fail_empty_file() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");

        let result = Config::new(temp_file.path().to_path_buf());

        assert!(
            result.is_err(),
            "Somehow there was valid json in this temp file"
        );
    }

    #[test]
    fn config_new_fail_nonexistent_file() {
        let result = Config::new(PathBuf::from("non/existent/pat"));

        assert!(result.is_err(), "Somehow a file with valid json exists");
    }

    #[test]
    fn config_new_fail_invalid_json() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": 2,
                "results_path": 2
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_path_buf());

        assert!(
            result.is_err(),
            "Somehow there was valid json in this temp file"
        );
    }

    #[test]
    fn config_fail_missing_required_field() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": 2
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_path_buf());

        assert!(
            result.is_err(),
            "Somehow there was valid json in this temp file"
        );
    }

    #[test]
    fn config_new_pass() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": ".",
                "results_path": "."
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result =
            Config::new(temp_file.path().to_path_buf()).expect("Failed to create struct somehow");
        let assert_config = Config {
            commands_path: PathBuf::from("."),
            results_path: PathBuf::from("."),
        };
        assert_eq!(result, assert_config);
    }
}
