use crate::interaction::command::{self, Command, parse_scenario};
use crate::interaction::config::Config;
use crate::threads::Itc;
use log::{info, warn};

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
                } => {}
            };
        }
    }
}
