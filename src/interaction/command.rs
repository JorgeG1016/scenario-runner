use anyhow::{Ok, Result};
use hex;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::PathBuf};

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
        send: Option<RawSendable>,
        expect_prefix: String,
        expect_exact: String,
        timeout: u64,
        delay: u64,
    },
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
        send: Option<Sendable>,
        expect_prefix: Vec<u8>,
        expect_exact: Vec<u8>,
        timeout: u64,
        delay: u64,
    },
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub command: Type,
    pub description: Option<String>,
}

pub fn parse_scenario(scenario: PathBuf) -> Result<Vec<Command>> {
    let file = File::open(scenario)?;
    let reader = BufReader::new(file);

    let raw_commands: Vec<RawCommand> = serde_json::from_reader(reader)?;
    let mut processed_commands: Vec<Command> = vec![];
    for raw_command in raw_commands {
        processed_commands.push(Command {
            command: match raw_command.command {
                RawType::Standard {
                    send,
                    expect_prefix,
                    expect_exact,
                    timeout,
                    delay,
                } => Type::Standard {
                    send: match send {
                        Some(raw_sendable) => match raw_sendable {
                            RawSendable::Hex { data } => Some(Sendable::Hex {
                                data: hex::decode(data)?,
                            }),
                            RawSendable::Text { data } => Some(Sendable::Text {
                                data: data.into_bytes(),
                            }),
                        },
                        None => None,
                    },
                    expect_prefix: expect_prefix.into_bytes(),
                    expect_exact: expect_exact.into_bytes(),
                    timeout: timeout,
                    delay: delay,
                },
            },
            description: raw_command.description,
        });
    }
    Ok(processed_commands)
}
