use anyhow::{Result, bail};
use hex;
use serde::Deserialize;
use std::time::Duration;
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawSendable {
    Hex { data: String },
    Text { data: String },
}

#[derive(Deserialize)]
#[serde(tag = "destination")]
enum RawDestination {
    Connection {
        send: Option<RawSendable>,
        expect_prefix: Option<String>,
        expect_exact: Option<String>,
        timeout: Option<u64>,
        delay: Option<u64>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommand {
    command: RawDestination,
    #[serde(default)]
    description: Option<String>,
}

impl RawCommand {
    fn validate(&self) -> Result<()> {
        match &self.command {
            RawDestination::Connection {
                expect_prefix,
                expect_exact,
                timeout,
                ..
            } => match (expect_prefix, expect_exact, timeout) {
                (Some(_), Some(_), Some(_)) | (None, None, None) => Ok(()),
                _ => bail!("expect_prefix, expect_exact, and timeout must all be provided or None"),
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Sendable {
    Hex { data: Vec<u8> },
    Text { data: Vec<u8> },
}

#[derive(Debug, PartialEq)]
pub enum Destination {
    Connection {
        send: Sendable,
        expect_prefix: Vec<u8>,
        expect_exact: Vec<u8>,
        timeout: Duration,
        delay: Duration,
    },
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub command: Destination,
    pub description: Option<String>,
}

impl TryFrom<RawSendable> for Sendable {
    type Error = anyhow::Error;
    fn try_from(value: RawSendable) -> Result<Self> {
        Ok(match value {
            RawSendable::Hex { data } => Sendable::Hex {
                data: hex::decode(data)?,
            },
            RawSendable::Text { data } => Sendable::Text {
                data: data.into_bytes(),
            },
        })
    }
}

impl TryFrom<RawCommand> for Command {
    type Error = anyhow::Error;
    fn try_from(value: RawCommand) -> Result<Self> {
        Ok(Command {
            command: match value.command {
                RawDestination::Connection {
                    send,
                    expect_prefix,
                    expect_exact,
                    timeout,
                    delay,
                } => Destination::Connection {
                    send: send
                        .map(Sendable::try_from)
                        .unwrap_or(Ok(Sendable::Text { data: Vec::new() }))?,
                    expect_prefix: expect_prefix
                        .map(|value| value.into_bytes())
                        .unwrap_or(Vec::new()),
                    expect_exact: expect_exact
                        .map(|value| value.into_bytes())
                        .unwrap_or(Vec::new()),
                    timeout: timeout
                        .map(Duration::from_secs)
                        .unwrap_or(Duration::from_secs(0)),
                    delay: delay
                        .map(Duration::from_secs)
                        .unwrap_or(Duration::from_secs(0)),
                },
            },
            description: value.description,
        })
    }
}

pub fn parse_scenario(scenario: &PathBuf) -> Result<Vec<Command>> {
    let file = File::open(scenario)?;
    let reader = BufReader::new(file);

    let raw_commands: Vec<RawCommand> = serde_json::from_reader(reader)?;
    let mut processed_commands: Vec<Command> = vec![];
    for raw_command in raw_commands {
        raw_command.validate()?;
        processed_commands.push(Command::try_from(raw_command)?);
    }
    Ok(processed_commands)
}

#[cfg(test)]
mod tests {

    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_scenario_single_pass() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            [
                {
                    "command": {
                        "destination": "Connection",
                        "send": {
                            "type": "Text",
                            "data": "Hello"
                        },
                        "expect_prefix": "This is the fixed sentence that always",
                        "expect_exact": "This is the fixed sentence that always appears",
                        "timeout": 240,
                        "delay": 0
                    }
                }
            ]
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write JSON");
        let scenario = temp_file.path().to_path_buf();

        let result = parse_scenario(&scenario).expect("Failed to parse scenario");
        let assert_command = Command {
            command: Destination::Connection {
                send: Sendable::Text {
                    data: Vec::from("Hello"),
                },
                expect_prefix: Vec::from("This is the fixed sentence that always"),
                expect_exact: Vec::from("This is the fixed sentence that always appears"),
                timeout: Duration::from_secs(240),
                delay: Duration::from_secs(0),
            },
            description: None,
        };

        assert_eq!(result[0], assert_command, "Failed to parse scenario");
    }

    #[test]
    fn parse_scenario_multiple_pass() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            [
                {
                    "command": {
                        "destination": "Connection",
                        "send": {
                            "type": "Text",
                            "data": "Hello"
                        },
                        "expect_prefix": "This is the fixed sentence that always",
                        "expect_exact": "This is the fixed sentence that always appears",
                        "timeout": 240,
                        "delay": 0
                    }
                },
                {
                    "command": {
                        "destination": "Connection",
                        "send": {
                            "type": "Hex",
                            "data": "deadbeef"
                        },
                        "expect_prefix": "This is the fixed sentence that always",
                        "expect_exact": "This is the fixed sentence that always appears",
                        "timeout": 240,
                        "delay": 0
                    }
                }
            ]
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write JSON");
        let scenario = temp_file.path().to_path_buf();

        let result = parse_scenario(&scenario).expect("Failed to parse scenario");
        let assert_text_command = Command {
            command: Destination::Connection {
                send: Sendable::Text {
                    data: Vec::from("Hello"),
                },
                expect_prefix: Vec::from("This is the fixed sentence that always"),
                expect_exact: Vec::from("This is the fixed sentence that always appears"),
                timeout: Duration::from_secs(240),
                delay: Duration::from_secs(0),
            },
            description: None,
        };
        let assert_hex_command = Command {
            command: Destination::Connection {
                send: Sendable::Hex {
                    data: vec![0xde, 0xad, 0xbe, 0xef],
                },
                expect_prefix: Vec::from("This is the fixed sentence that always"),
                expect_exact: Vec::from("This is the fixed sentence that always appears"),
                timeout: Duration::from_secs(240),
                delay: Duration::from_secs(0),
            },
            description: None,
        };

        assert_eq!(result[0], assert_text_command, "Failed to parse scenario");
        assert_eq!(result[1], assert_hex_command, "Failed to parse scenario");
    }

    #[test]
    fn parse_scenario_invalid_command() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let raw_json = r#"
            [
                {
                    "command": {
                        "destination": "Connection",
                        "send": {
                            "type": "Text",
                            "data": "Hello"
                        },
                        "expect_prefix": "This is the fixed sentence that always",
                        "expect_exact": "This is the fixed sentence that always appears",
                        "timeout": 240,
                        "delay": 0
                    }
                },
                {
                    "command": {
                        "destination": "Connection",
                        "send": {
                            "type": "Hex",
                            "data": "deadbeef"
                        },
                        "expect_prefix": "This is the fixed sentence that always",
                        "timeout": 240,
                        "delay": 0
                    }
                }
            ]
            "#;
        temp_file
            .write_all(raw_json.as_bytes())
            .expect("Failed to write JSON");
        let scenario = temp_file.path().to_path_buf();

        let result = parse_scenario(&scenario);
        assert!(result.is_err(), "Somehow the JSON was actually valid");
    }
}
