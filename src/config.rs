use anyhow::{Ok, Result};
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::path::PathBuf;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ConnectionType {
    Usb { port: String, baud_rate: u32 },
    Tcp { address: String, port: u32 },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    commands_path: String,
    #[serde(default)]
    results_path: Option<String>,
    interface: ConnectionType,
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub commands_path: PathBuf,
    pub results_path: PathBuf,
    pub interface: ConnectionType,
}

impl Config {
    pub fn new(config_file_string: String) -> Result<Config> {
        let config_file_path = PathBuf::from(config_file_string);
        if !config_file_path.exists() {
            return Err(anyhow::anyhow!("Config file does not exist"));
        }

        let config_reader = io::BufReader::new(File::open(config_file_path)?);
        let parsed_raw_config: RawConfig = serde_json::from_reader(config_reader)?;
        let mut temp_path = PathBuf::from(parsed_raw_config.commands_path);
        let processed_config = Config {
            commands_path: temp_path.clone(),
            interface: parsed_raw_config.interface,
            results_path: match parsed_raw_config.results_path {
                Some(value) => PathBuf::from(value),
                None => {
                    temp_path.push("results");
                    temp_path
                }
            },
        };
        Ok(processed_config)
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

        let result = Config::new(temp_file.path().to_str().unwrap().to_string());

        assert!(
            result.is_err(),
            "Somehow there was valid json in this temp file"
        );
    }

    #[test]
    fn config_new_fail_nonexistent_file() {
        let result = Config::new("non/existent/pat".to_string());

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

        let result = Config::new(temp_file.path().to_str().unwrap().to_string());
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

        let result = Config::new(temp_file.path().to_str().unwrap().to_string());

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
                "unknown_field": "."
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_str().unwrap().to_string());

        assert!(
            result.is_err(),
            "Somehow there was valid json in this temp file"
        );
    }

    #[test]
    fn config_new_fail_interface_field_mismatch() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "commands_path": ".",
                "results_path": ".",
                "interface": {
                    "type": "Usb",
                    "address": "test:test"
                }
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_str().unwrap().to_string());

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
                "interface": {
                    "type": "Tcp",
                    "address": "test",
                    "port": 8080
                }
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            commands_path: PathBuf::from("."),
            results_path: PathBuf::from(".").join("results"),
            interface: ConnectionType::Tcp {
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
                "interface": {
                    "type": "Tcp",
                    "address": "test",
                    "port": 8080
                }
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            commands_path: PathBuf::from("."),
            results_path: PathBuf::from("."),
            interface: ConnectionType::Tcp {
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
                "interface": {
                    "type": "Usb",
                    "port": "test",
                    "baud_rate": 115200
                }
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            commands_path: PathBuf::from("."),
            results_path: PathBuf::from("."),
            interface: ConnectionType::Usb {
                port: String::from("test"),
                baud_rate: 115200,
            },
        };
        assert_eq!(result, assert_config);
    }
}
