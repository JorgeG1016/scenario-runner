use clap::Parser;
use colored::*;
use config::Config;
use connection::*;

mod config;
mod connection;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value = "./config.json")]
    config_file: String,
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
