use crate::interaction::command::{Command, parse_scenario};
use crate::interaction::config::Config;
use log::{info, warn};

pub fn thread(config: Config) {
    info!("Starting Scenario Handler Thread");
    for scenario in config.scenarios {
        if !scenario.is_file() {
            warn!("{} does not exist, skipping", scenario.display());
            continue;
        }

        let scenario_commands = parse_scenario(scenario);
        print!("Here: {:?}", scenario_commands);
    }
}
