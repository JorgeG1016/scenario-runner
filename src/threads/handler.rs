use log::{info, warn};
use crate::interaction::config::Config;

pub fn thread(config: Config) { 
    info!("Starting Scenario Handler Thread");
    for scenario in config.scenarios{
        if !scenario.is_file() {
            warn!("{} does not exist, skipping", scenario.display());
            continue;
        }
    }
}