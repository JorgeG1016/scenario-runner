use anyhow::{Ok, Result};
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::path::PathBuf;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ConnectionType {
    Usb { port: String, baud_rate: u32 },
    Tcp { address: String, port: u16 },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    scenarios_location: Option<String>,
    #[serde(default)]
    results_location: Option<String>,
    connection: ConnectionType,
    scenarios: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub scenarios_location: PathBuf,
    pub results_location: PathBuf,
    pub connection: ConnectionType,
    pub scenarios: Vec<PathBuf>,
}

impl Config {
    pub fn new(config_file_string: String) -> Result<Self> {
        let config_file_path = PathBuf::from(config_file_string);
        if !config_file_path.exists() {
            return Err(anyhow::anyhow!("Specified Config file does not exist"));
        }

        // Process the fields from the raw struct into the final output
        let config_reader = io::BufReader::new(File::open(config_file_path)?);
        let parsed_raw_config: RawConfig = serde_json::from_reader(config_reader)?;
        let temp_path = match parsed_raw_config.scenarios_location {
            Some(value) => PathBuf::from(value),
            None => PathBuf::from("."),
        };
        let processed_config = Config {
            scenarios_location: temp_path.clone(),
            connection: parsed_raw_config.connection,
            results_location: match parsed_raw_config.results_location {
                Some(value) => PathBuf::from(value),
                None => temp_path.clone(),
            },
            scenarios: parsed_raw_config
                .scenarios
                .iter()
                .map(|s| temp_path.join(s))
                .collect(),
        };

        print!("{:?}", processed_config.scenarios);

        Ok(processed_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
                "scenarios_location": 2
                "results_location": 2
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
                "scenarios_location": 2
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
                "scenarios_location": ".",
                "results_location": ".",
                "unknown_field": ".",
                "scenarios": [
                    "scenario1",
                    "scenario2",
                ]
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
    fn config_new_fail_connection_field_mismatch() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "scenarios_location": ".",
                "results_location": ".",
                "connection": {
                    "type": "Usb",
                    "address": "test:test"
                },
                "scenarios": [
                    "scenario1",
                    "scenario2",
                ]
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
    fn config_new_pass_without_results_location() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "scenarios_location": ".",
                "connection": {
                    "type": "Tcp",
                    "address": "test",
                    "port": 8080
                },
                "scenarios": [
                    "scenario1",
                    "scenario2"
                ]
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            scenarios_location: PathBuf::from("."),
            results_location: PathBuf::from("."),
            connection: ConnectionType::Tcp {
                address: String::from("test"),
                port: 8080,
            },
            scenarios: vec![PathBuf::from("./scenario1"), PathBuf::from("./scenario2")],
        };
        assert_eq!(result, assert_config);
    }

    #[test]
    fn config_new_pass_without_scenarios_location() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "results_location": ".",
                "connection": {
                    "type": "Tcp",
                    "address": "test",
                    "port": 8080
                },
                "scenarios": [
                    "scenario1",
                    "scenario2"
                ]
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            scenarios_location: PathBuf::from("."),
            results_location: PathBuf::from("."),
            connection: ConnectionType::Tcp {
                address: String::from("test"),
                port: 8080,
            },
            scenarios: vec![PathBuf::from("./scenario1"), PathBuf::from("./scenario2")],
        };
        assert_eq!(result, assert_config);
    }

    #[test]
    fn config_new_pass_tcp_io() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "scenarios_location": ".",
                "results_location": ".",
                "connection": {
                    "type": "Tcp",
                    "address": "test",
                    "port": 8080
                },
                "scenarios": [
                    "scenario1",
                    "scenario2"
                ]
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            scenarios_location: PathBuf::from("."),
            results_location: PathBuf::from("."),
            connection: ConnectionType::Tcp {
                address: String::from("test"),
                port: 8080,
            },
            scenarios: vec![PathBuf::from("./scenario1"), PathBuf::from("./scenario2")],
        };
        assert_eq!(result, assert_config);
    }

    #[test]
    fn config_new_pass_usb_io() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "scenarios_location": ".",
                "results_location": ".",
                "connection": {
                    "type": "Usb",
                    "port": "test",
                    "baud_rate": 115200
                },
                "scenarios": [
                    "scenario1",
                    "scenario2"
                ]
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");
        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            scenarios_location: PathBuf::from("."),
            results_location: PathBuf::from("."),
            connection: ConnectionType::Usb {
                port: String::from("test"),
                baud_rate: 115200,
            },
            scenarios: vec![PathBuf::from("./scenario1"), PathBuf::from("./scenario2")],
        };
        assert_eq!(result, assert_config);
    }

    #[test]
    fn config_new_pass_without_any_location() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            {
                "connection": {
                    "type": "Tcp",
                    "address": "test",
                    "port": 8080
                },
                "scenarios": [
                    "scenario1",
                    "scenario2"
                ]
            }
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write to temp file");

        let result = Config::new(temp_file.path().to_str().unwrap().to_string())
            .expect("Somehow a valid struct wasn't created");
        let assert_config = Config {
            scenarios_location: PathBuf::from("."),
            results_location: PathBuf::from("."),
            connection: ConnectionType::Tcp {
                address: String::from("test"),
                port: 8080,
            },
            scenarios: vec![PathBuf::from("./scenario1"), PathBuf::from("./scenario2")],
        };
        assert_eq!(result, assert_config);
    }
}
