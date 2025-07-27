use anyhow::Result;
use clap::Parser;
use env_logger::{self, TimestampPrecision};
use log::info;
use threads::controller;

mod connection;
mod interaction;
mod threads;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value = "./config.json")]
    config_file: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(TimestampPrecision::Millis))
        .write_style(env_logger::WriteStyle::Always)
        .init();

    info!("Parsing config file");
    let args = Args::parse();

    // Don't actually spawn a thread but can be spawned as a separate thread from main if needed
    controller::thread(args.config_file)?;

    info!("Scenario Runner has finished running");
    Ok(())
}
