use anyhow::{Error, Result};
use hex;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::PathBuf};
use std::time::Duration;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawSendable {
    Hex { data: String },
    Text { data: String },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawType {
    Standard {
        send: RawSendable,
        expect_prefix: String,
        expect_exact: String,
        timeout: u64,
        delay: u64,
    },
    Wait {
        expect_prefix: String,
        expect_exact: String,
        timeout: u64,
        delay: u64,
    },
    WriteOnly {
        send: RawSendable,
        delay: u64
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommand {
    command: RawType,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Sendable {
    Hex { data: Vec<u8> },
    Text { data: Vec<u8> },
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Standard {
        send: Sendable,
        expect_prefix: Vec<u8>,
        expect_exact: Vec<u8>,
        timeout: Duration,
        delay: Duration,
    },
    Wait {
        expect_prefix: Vec<u8>,
        expect_exact: Vec<u8>,
        timeout: Duration,
        delay: Duration
    },
    WriteOnly {
        send: Sendable,
        delay: Duration,
    }
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub command: Type,
    pub description: Option<String>,
}

impl TryFrom<RawSendable> for Sendable {
    type Error = anyhow::Error;
    fn try_from(value: RawSendable) -> Result<Self> {
        Ok(match value {
            RawSendable::Hex { data } => Sendable::Hex { data: hex::decode(data)? },
            RawSendable::Text { data } => Sendable::Text { data: data.into_bytes() }
        })
    }
} 

impl TryFrom<RawCommand> for Command {
    type Error = anyhow::Error;
    fn try_from(value: RawCommand) -> Result<Self> {
        Ok(Command {
            command: match value.command 
            {
                RawType::Standard 
                { 
                    send, 
                    expect_prefix, 
                    expect_exact, 
                    timeout, 
                    delay 
                } => Type::Standard { 
                    send: Sendable::try_from(send)?,
                    expect_prefix: expect_prefix.into_bytes(), 
                    expect_exact: expect_exact.into_bytes(), 
                    timeout: Duration::from_secs(timeout), 
                    delay: Duration::from_secs(delay)
                },
                RawType::Wait 
                { 
                    expect_prefix, 
                    expect_exact, 
                    timeout, 
                    delay 
                } => Type::Wait { 
                    expect_prefix: expect_prefix.into_bytes(), 
                    expect_exact: expect_exact.into_bytes(), 
                    timeout: Duration::from_secs(timeout), 
                    delay: Duration::from_secs(delay) 
                },
                RawType::WriteOnly 
                { 
                    send, 
                    delay 
                } => Type::WriteOnly 
                { 
                    send: Sendable::try_from(send)?, 
                    delay: Duration::from_secs(delay) 
                }
            },
            description: value.description
        })
    }
}

pub fn parse_scenario(scenario: &PathBuf) -> Result<Vec<Command>> {
    let file = File::open(scenario)?;
    let reader = BufReader::new(file);

    let raw_commands: Vec<RawCommand> = serde_json::from_reader(reader)?;
    let mut processed_commands: Vec<Command> = vec![];
    for raw_command in raw_commands {
        processed_commands.push(Command::try_from(raw_command)?);
    }
    Ok(processed_commands)
}
