use anyhow::{Result, bail};
use crossbeam::channel::{self, Receiver, Sender};
use log::info;
use std::collections::HashMap;
use std::thread;

use crate::connection::Communicate;
use crate::connection::tcp::Connection as TcpConnection;
use crate::connection::usb::Connection as UsbConnection;
use crate::interaction::config::{Config, ConnectionType};
use crate::threads::itc::{Endpoints, Message};
use crate::threads::{handler, runner};

struct Link {
    controller_end: Endpoints,
    thread_end: Endpoints,
}

struct Controller {
    registry: HashMap<String, Link>,
    inbox: Receiver<Message>,
    outbox: Sender<Message>,
    source: String,
}

impl Controller {
    fn new(source: String) -> Self {
        let (outbox, inbox) = channel::unbounded();
        Controller {
            registry: HashMap::new(),
            outbox,
            inbox,
            source,
        }
    }

    fn add_link(&mut self, thread_name: String) {
        let (thread_tx, thread_rx) = channel::unbounded::<Message>();
        let controller_end = Endpoints::new(self.source.clone(), thread_tx, self.inbox.clone());
        let thread_end = Endpoints::new(thread_name.clone(), self.outbox.clone(), thread_rx);
        //Not a big deal, but want to thread name at least, so no clone
        self.registry.insert(
            thread_name,
            Link {
                controller_end,
                thread_end,
            },
        );
    }

    fn get_thread_endpoint(&self, thread_name: &String) -> Result<&Endpoints> {
        match self.registry.get(thread_name) {
            Some(value) => Ok(&value.thread_end),
            None => bail!("No key with that value in registry"),
        }
    }
}

pub fn thread(config_file: String) -> Result<()> {
    let current_config = Config::new(config_file)?;

    info!("Connecting using specified configuration");
    let mut opened_connection = open_connection(current_config.connection)?;

    //
    let thread_ids: Vec<String> = vec![String::from("handler"), String::from("runner")];

    // let (controller_send, controller_receive) = mpsc::channel();
    // // handler thread is sort of the hub, needs to be connected to other threads
    // let (handler_tx, handler_rx) = mpsc::channel();
    // let (runner_tx, runner_rx) = mpsc::channel();

    // let handler_channels = itc::Itc::new(handler_tx, runner_rx);
    // let runner_channels = itc::Itc::new(runner_tx, handler_rx);

    // let handler_handle =
    //     thread::spawn(move || handler::thread(current_config.scenarios, handler_channels));
    // let runner_handle =
    //     thread::spawn(move || runner::thread(&mut opened_connection, runner_channels));

    // let _ = handler_handle.join();
    // let _ = runner_handle.join();
    Ok(())
}

fn open_connection(
    connection_type: ConnectionType,
) -> Result<Box<dyn Communicate + Send + 'static>> {
    match connection_type {
        ConnectionType::Tcp { address, port } => Ok(Box::new(TcpConnection::new(address, port)?)),
        ConnectionType::Usb { port, baud_rate } => {
            Ok(Box::new(UsbConnection::new(port, baud_rate)?))
        }
    }
}
