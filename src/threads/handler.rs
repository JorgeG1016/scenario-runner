use super::itc::{Itc, Message};
use crate::interaction::command::{self, Sendable, parse_scenario};
use log::{error, info, trace, warn};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

pub fn thread(scenarios: Vec<PathBuf>, runner_channels: Itc) {
    info!("Starting Scenario Handler Thread!");
    'scenario_loop: for scenario in scenarios {
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
    let _ = runner_channels.send(Message::StopRunning);
    info!("Stopping Scenario Handler Thread!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, path::PathBuf, sync::mpsc::channel, vec};
    use tempfile::NamedTempFile;

    fn setup() -> (Itc, Itc) {
        let (test_tx, test_rx) = channel();
        let (thread_tx, thread_rx) = channel();
        (
            Itc::new(test_tx, thread_rx),
            Itc::new(thread_tx, test_rx),
        )
    }

    #[test]
    fn thread_no_scenarios() {
        let (unit_channel, thread_channel) = setup();

        let handle = thread::spawn(move || thread(Vec::new(), thread_channel));
        let received_message = unit_channel.receive_timeout(Duration::from_secs(5)).expect("Should've received a runner stop message");

        assert!(matches!(received_message, Message::StopRunning), "Unexpectedly received something else");
        assert!(handle.join().is_ok(), "Thread joined with fail")
    }

    #[test]
    fn thread_scenario_not_a_file() {
        let (unit_channel, thread_channel) = setup();

        let handle = thread::spawn(move || thread(vec![PathBuf::from(".")], thread_channel));
        let received_message = unit_channel.receive_timeout(Duration::from_secs(5)).expect("Should've received a runner stop message");

        assert!(matches!(received_message, Message::StopRunning), "Unexpectedly received something else");
        assert!(handle.join().is_ok(), "Thread joined with fail")
    }

    #[test]
    fn thread_invalid_scenario() {
        let (unit_channel, thread_channel) = setup();
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
                        "delay": 0,
                    }
                }
            ]
            "#;
        temp_file.write_all(raw_json.as_bytes()).expect("Failed to write dummy scenario");
        let scenarios = vec![temp_file.path().to_path_buf()];

        let handle = thread::spawn(move || thread(scenarios, thread_channel));
        let received_message = unit_channel.receive_timeout(Duration::from_secs(5)).expect("Should've received a runner stop message");

        assert!(matches!(received_message, Message::StopRunning), "Unexpectedly received something else");
        assert!(handle.join().is_ok(), "Thread joined with fail")
    }
}