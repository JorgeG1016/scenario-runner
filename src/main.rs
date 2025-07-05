use std::io::Result;
use std::path::PathBuf;

use clap::Parser;
use colored::*;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value = "./config.json")]
    config_file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.config_file.exists() {
        println!(
            "{}: Found the file {}",
            "SUCCESS".green(),
            args.config_file.display()
        );
    } else {
        println!("{}: {}", "ERROR".red(), "File does not exist");
    }
    Ok(())
}
