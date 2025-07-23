use crate::interaction::command::{self, parse_scenario, Command, Sendable};
use crate::interaction::config::Config;
use super::itc::{Itc, Messages};
use log::{info, warn};
use std::time::Instant;
use std::thread;

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

        for command in scenario_commands {
            match command.command {
                command::Type::Standard {
                    send,
                    expect_prefix,
                    expect_exact,
                    timeout,
                    delay,
                } => {
                    let data = match send {
                        Sendable::Hex { data } => data,
                        Sendable::Text { data } => data,
                    };
                    thread::sleep(delay);
                    runner_channels.send_channel.send(Messages::SendData { data: data });
                    let start_time = Instant::now();
                    while Instant::now() - start_time < timeout {
                        let received_data = runner_channels.receive_channel.recv();
                        
                    }
                },
                command::Type::Wait { 
                    expect_prefix, 
                    expect_exact, 
                    timeout, 
                    delay 
                } => {

                },
                command::Type::WriteOnly { 
                    send, 
                    delay 
                } => {

                }
            };
        }
    }
}
