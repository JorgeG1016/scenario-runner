use super::itc::{Itc, Message};
use crate::interaction::command::{self, Sendable, parse_scenario};
use crate::interaction::config::Config;
use log::{error, info, trace, warn};
use std::thread;
use std::time::{Duration, Instant};

pub fn thread(config: Config, runner_channels: Itc) {
    info!("Starting Scenario Handler Thread!");
    'scenario_loop: for scenario in config.scenarios {
        if !scenario.is_file() {
            warn!("{} does not exist, skipping", scenario.display());
            continue;
        }

        let scenario_commands = match parse_scenario(&scenario) {
            Ok(commands) => commands,
            Err(_) => {
                warn!("{} could not be parsed, skipping", scenario.display());
                continue;
            }
        };

        for (cnt, command) in scenario_commands.into_iter().enumerate() {
            match command.command {
                command::Destination::Connection {
                    send,
                    expect_exact,
                    expect_prefix,
                    timeout,
                    delay,
                } => {
                    let data = match send {
                        Sendable::Hex { data } => data,
                        Sendable::Text { data } => data,
                    };
                    thread::sleep(delay);
                    let start_sequence = vec![Message::SendData { data }, Message::StartDataStream];
                    trace!("Sending command {} in scenario {}", cnt, scenario.display());
                    if runner_channels.send_all(start_sequence).is_err() {
                        warn!(
                            "Command {} in {} could not be sent, skipping",
                            cnt,
                            scenario.display()
                        );
                    }
                    let start_time = Instant::now();
                    while Instant::now() - start_time < timeout {
                        let elapsed_time = Instant::now() - start_time;
                        let remaining_time = timeout
                            .checked_sub(elapsed_time)
                            .unwrap_or(Duration::from_secs(0));

                        if remaining_time.is_zero() {
                            trace!(
                                "Command timed out, expected prefix or response was not received"
                            );
                            break;
                        }

                        if let Ok(message) = runner_channels.receive_timeout(remaining_time) {
                            if !expect_prefix.is_empty() {
                                match message {
                                    Message::DataReceived { data, .. } => {
                                        if data.starts_with(&expect_prefix) {
                                            if data == expect_exact {
                                                trace!("Found exact response");
                                                break;
                                            } else {
                                                trace!(
                                                    "Found expected prefix, but response didn't match"
                                                );
                                                break;
                                            }
                                        }
                                    }
                                    Message::SendError | Message::ReceiveError => {
                                        error!(
                                            "Something went wrong with the connection, shutting down program"
                                        );
                                        let _ = runner_channels.send(Message::StopRunning);
                                        break 'scenario_loop;
                                    }
                                    _ => {
                                        warn!("Received something unexpected from runner")
                                    }
                                }
                            }
                        }
                    }
                }
            };
        }
    }
    info!("Stopping Scenario Handler Thread!");
}
