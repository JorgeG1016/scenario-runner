use clap::Parser;
use connection::Communicate;
use connection::tcp::Connection as TcpConnection;
use connection::usb::Connection as UsbConnection;
use env_logger::{self, TimestampPrecision};
use interaction::config::{Config, ConnectionType};
use log::{error, info};
use std::thread;
use threads::runner_thread;
use threads::handler_thread;

mod connection;
mod interaction;
mod threads;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value = "./config.json")]
    config_file: String,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(TimestampPrecision::Millis))
        .write_style(env_logger::WriteStyle::Always)
        .init();

    info!("Parsing config file");
    let args = Args::parse();
    let current_config = match Config::new(args.config_file) {
        Ok(config) => config,
        Err(msg) => {
            error!("Issue with config file [{msg}]");
            return;
        }
    };

    info!("Setting up connection");
    //Cloning since the complete struct still needs to be passed to the handler thread
    let connection = current_config.connection.clone();
    let mut opened_connection: Box<dyn Communicate + Send + 'static> = match connection {
        ConnectionType::Tcp { address, port } => {
            let tcp_connection = match TcpConnection::new(address, port) {
                Ok(new_connection) => new_connection,
                Err(msg) => {
                    error!("Issue opening TCP connection [{msg}]");
                    return;
                }
            };
            Box::new(tcp_connection)
        }
        ConnectionType::Usb { port, baud_rate } => {
            let usb_connection = match UsbConnection::new(port, baud_rate) {
                Ok(new_connection) => new_connection,
                Err(msg) => {
                    error!("Issue opening USB connection [{msg}]");
                    return;
                }
            };
            Box::new(usb_connection)
        }
    };
    

    let handler_handle = thread::spawn(move || handler_thread(current_config));

    let runner_handle = thread::spawn(move || runner_thread(&mut opened_connection));
    let _ = handler_handle.join();
    let _ = runner_handle.join();
}
