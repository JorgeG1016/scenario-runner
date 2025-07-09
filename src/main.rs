use clap::Parser;
use colored::*;
use config::Config;
use std::path::PathBuf;

mod config;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value = "./config.json")]
    config_file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let current_config = match Config::new(args.config_file) {
        Ok(config) => config,
        Err(error) => {
            println!("{}: {}", "ERROR".red().bold(), error);
            return;
        }
    };
    println!("{current_config:?}");
}
