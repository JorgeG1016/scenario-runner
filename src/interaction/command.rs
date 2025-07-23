use anyhow::{Result, bail};
use hex;
use serde::Deserialize;
use std::any;
use std::{fs::File, io::BufReader, path::PathBuf};
use std::time::Duration;

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
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommand {
    command: RawDestination,
    #[serde(default)]
    description: Option<String>,
}

impl RawCommand {
    fn validate(&self) -> Result<()>{
        match &self.command {
            RawDestination::Connection { expect_prefix, expect_exact, .. } => {
                match (expect_prefix, expect_exact) { 
                    (Some(_), Some(_)) | (None, None) => Ok(()),
                    _ => bail!("expect_prefix and expect_exact must both be provided or None")
                }
            }
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
    }
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
                RawDestination::Connection 
                { 
                    send, 
                    expect_prefix, 
                    expect_exact, 
                    timeout, 
                    delay 
                } => Destination::Connection { 
                    send: send.map(|value| Sendable::try_from(value)).unwrap_or(Ok(Sendable::Text { data: Vec::new() }))?,
                    expect_prefix: expect_prefix.map(|value| value.into_bytes()).unwrap_or(Vec::new()), 
                    expect_exact: expect_exact.map(|value| value.into_bytes()).unwrap_or(Vec::new()), 
                    timeout: timeout.map(|value| Duration::from_secs(value)).unwrap_or(Duration::from_secs(0)), 
                    delay: delay.map(|value| Duration::from_secs(value)).unwrap_or(Duration::from_secs(0))
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
        raw_command.validate()?;
        processed_commands.push(Command::try_from(raw_command)?);
    }
    Ok(processed_commands)
}
