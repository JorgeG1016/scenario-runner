use anyhow::{Result, bail};
use crossbeam::channel::{self, Receiver, Sender};
use log::info;
use std::collections::HashMap;
use std::thread;

use crate::connection::Communicate;
use crate::connection::tcp::Connection as TcpConnection;
use crate::connection::usb::Connection as UsbConnection;
use crate::interaction::config::{Config, ConnectionType};
use crate::threads::itc::{Endpoints, Event, Message};
use crate::threads::{handler, runner};

struct Controller {
    registry: HashMap<String, Endpoints>,
    mailbox: Endpoints,
}

impl Controller {
    fn new(source: String) -> Self {
        let (send_channel, receive_channel) = channel::unbounded();
        let endpoint = Endpoints::new(source, send_channel, receive_channel);

        Controller {
            registry: HashMap::new(),
            mailbox: endpoint,
        }
    }

    fn add_link(&mut self, thread_name: String) -> Endpoints {
        let (thread_tx, thread_rx) = channel::unbounded::<Message>();
        let (controller_tx, controller_rx) = self.mailbox.get_channels();
        let controller_end = Endpoints::new(self.mailbox.get_source(), thread_tx, controller_rx);
        let thread_end = Endpoints::new(thread_name.clone(), controller_tx, thread_rx);
        //Not a big deal, but want to drop thread name at least, so no clone
        self.registry.insert(thread_name, controller_end);
        thread_end
    }

    fn wait_on_inbox(&self) -> Result<Message> {
        Ok(self.mailbox.receive_blocking()?)
    }

    fn send_to_thread(&self, thread_name: String, data: Event) -> Result<()> {
        match self.registry.get(&thread_name) {
            Some(value) => value.send(data)?,
            None => bail!("Thread does not exist in registry"),
        };
        Ok(())
    }
}

pub fn thread(config_file: String) -> Result<()> {
    let current_config = Config::new(config_file)?;

    info!("Connecting using specified configuration");
    let mut opened_connection = open_connection(current_config.connection)?;

    let mut hub = Controller::new("controller".to_string());
    let handler_endpoint = hub.add_link("handler".to_string());
    let runner_endpoint = hub.add_link("runner".to_string());

    let handler_handle =
        thread::spawn(move || handler::thread(current_config.scenarios, handler_endpoint));
    let runner_handle =
        thread::spawn(move || runner::thread(&mut opened_connection, runner_endpoint));

    let _ = handler_handle.join();
    let _ = runner_handle.join();
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

fn process_message(hub: Controller) -> Result<()> {
    loop {
        let message = hub.wait_on_inbox()?;
        match message.origin.as_str() {
            "handler" => {}
            "runner" => {}
            _ => {}
        };
    }
}

fn process_handler_message(endpoints: &Endpoints, event: Event) {
    match event {}
}
