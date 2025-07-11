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
#[serde(tag = "type")]
pub enum ConfigIOType {
    USB { port: String, baud_rate: u32 },
    TCP { address: String, port: u32 },
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    commands_path: PathBuf,
    #[serde(default)]
    results_path: Option<PathBuf>,
    sequence: Vec<PathBuf>,
    io_interface: ConfigIOType,
}

impl Config {
    pub fn new(config_file_path: PathBuf) -> Result<Self, ConfigErrors> {
        if !config_file_path.exists() {
            return Err(ConfigErrors::DoesNotExist);
        }

        let config_file = File::open(config_file_path)?;
        let config_reader = io::BufReader::new(config_file);
        let mut parsed_config: Config = serde_json::from_reader(config_reader)?;
        if parsed_config.results_path.is_none() {
            let mut temp_path = parsed_config.commands_path.clone();
            temp_path.push("results");
            parsed_config.results_path = Some(temp_path);
        }
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
    fn config_new_fail_missing_required_field() {
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
    fn config_new_fail_unknown_field_present() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": ".",
                "results_path": ".",
                "sequence": ["test.txt", "test.txt"],
                "unknown_field": "."
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
    fn config_new_fail_io_interface_field_mismatch() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": ".",
                "results_path": ".",
                "sequence": ["test1.txt", "test2.txt"],
                "io_interface": {
                    "type": "USB",
                    "address": "test:test"
                }
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
    fn config_new_pass_without_results_path() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": ".",
                "sequence": ["test1.txt", "test2.txt"],
                "io_interface": {
                    "type": "TCP",
                    "address": "test",
                    "port": 8080
                }
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result =
            Config::new(temp_file.path().to_path_buf()).expect("Failed to create struct somehow");
        let assert_config = Config {
            commands_path: PathBuf::from("."),
            results_path: Some(PathBuf::from(".").join("results")),
            sequence: vec![PathBuf::from("test1.txt"), PathBuf::from("test2.txt")],
            io_interface: ConfigIOType::TCP {
                address: String::from("test"),
                port: 8080,
            },
        };
        assert_eq!(result, assert_config);
    }

    #[test]
    fn config_new_pass_tcp_io() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": ".",
                "results_path": ".",
                "sequence": ["test1.txt", "test2.txt"],
                "io_interface": {
                    "type": "TCP",
                    "address": "test",
                    "port": 8080
                }
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result =
            Config::new(temp_file.path().to_path_buf()).expect("Failed to create struct somehow");
        let assert_config = Config {
            commands_path: PathBuf::from("."),
            results_path: Some(PathBuf::from(".")),
            sequence: vec![PathBuf::from("test1.txt"), PathBuf::from("test2.txt")],
            io_interface: ConfigIOType::TCP {
                address: String::from("test"),
                port: 8080,
            },
        };
        assert_eq!(result, assert_config);
    }

    #[test]
    fn config_new_pass_usb_io() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": ".",
                "results_path": ".",
                "sequence": ["test1.txt", "test2.txt"],
                "io_interface": {
                    "type": "USB",
                    "port": "test",
                    "baud_rate": 115200
                }
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result =
            Config::new(temp_file.path().to_path_buf()).expect("Failed to create struct somehow");
        let assert_config = Config {
            commands_path: PathBuf::from("."),
            results_path: Some(PathBuf::from(".")),
            sequence: vec![PathBuf::from("test1.txt"), PathBuf::from("test2.txt")],
            io_interface: ConfigIOType::USB {
                port: String::from("test"),
                baud_rate: 115200,
            },
        };
        assert_eq!(result, assert_config);
    }
}
