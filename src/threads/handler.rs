use super::itc::{Itc, Messages};
use crate::interaction::command::{self, Sendable, parse_scenario};
use crate::interaction::config::Config;
use log::{info, warn};
use std::thread;
use std::time::Instant;

pub fn thread(config: Config, runner_channels: Itc) {
    info!("Starting Scenario Handler Thread");
    for scenario in config.scenarios {
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
                    timeout,
                    delay,
                    ..
                } => {
                    let data = match send {
                        Sendable::Hex { data } => data,
                        Sendable::Text { data } => data,
                    };
                    thread::sleep(delay);
                    if runner_channels
                        .send_channel
                        .send(Messages::SendData { data })
                        .is_err()
                    {
                        warn!(
                            "Command {} in {} could not be sent, skipping",
                            cnt,
                            scenario.display()
                        );
                    }
                    let start_time = Instant::now();
                    while Instant::now() - start_time < timeout {
                        let _received_data = runner_channels.receive_channel.recv();
                    }
                }
            };
        }
    }
}
