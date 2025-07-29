use anyhow::{Result, bail};
use crossbeam::channel::{self, Receiver, Sender};
use log::info;
use std::collections::HashMap;
use std::thread;
use chrono::{DateTime, Local};
use std::time::Duration;

use crate::connection::Communicate;
use crate::connection::tcp::Connection as TcpConnection;
use crate::connection::usb::Connection as UsbConnection;
use crate::interaction::config::{Config, ConnectionType};
use crate::threads::{handler, runner};

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    StartDataStream,
    StopDataStream,
    SendData {
        data: Vec<u8>,
    },
    DataReceived {
        timestamp: DateTime<Local>,
        data: Vec<u8>,
        data_length: usize,
    },
    StopRunning,
    SendError,
    ReceiveError,
    HandlerReceivedData {
        timestamp: DateTime<Local>,
        data: Vec<u8>,
        data_length: usize
    },
    HandlerSendData {
        data: Vec<u8>
    }

}

#[derive(Debug, Clone)]
pub struct Endpoints {
    send_channel: Sender<Message>,
    receive_channel: Receiver<Message>,
}

impl Endpoints {
    pub fn new(
        send_channel: Sender<Message>,
        receive_channel: Receiver<Message>,
    ) -> Self {
        Self {
            send_channel,
            receive_channel,
        }
    }

    pub fn send_all(&self, messages: Vec<Message>) -> Result<()> {
        for message in messages {
            self.send_channel.send(message)?;
        }
        Ok(())
    }

    pub fn try_receive_all(&self) -> Result<Vec<Message>> {
        let mut messages: Vec<Message> = Vec::new();
        while let Ok(message) = self.receive_channel.try_recv() {
            messages.push(message);
        }
        Ok(messages)
    }

    pub fn receive_timeout(&self, timeout: Duration) -> Result<Message> {
        Ok(self.receive_channel.recv_timeout(timeout)?)
    }

    pub fn receive_blocking(&self) -> Result<Message> {
        Ok(self.receive_channel.recv()?)
    }

    pub fn send(&self, message: Message) -> Result<()> {
        Ok(self.send_channel.send(message)?)
    }

    pub fn get_channels(&self) -> (Sender<Message>, Receiver<Message>) {
        (self.send_channel.clone(), self.receive_channel.clone())
    }
}


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Identifier {
    Handler,
    Runner
}
struct Controller {
    registry: HashMap<Identifier, Endpoints>,
    mailbox: Endpoints,
}

impl Controller {
    fn new() -> Self {
        let (send_channel, receive_channel) = channel::unbounded();
        let endpoint = Endpoints::new(send_channel, receive_channel);

        Controller {
            registry: HashMap::new(),
            mailbox: endpoint,
        }
    }

    fn add_link(&mut self, identifier: Identifier) -> Endpoints {
        let (thread_tx, thread_rx) = channel::unbounded::<Message>();
        let (controller_tx, controller_rx) = self.mailbox.get_channels();
        let controller_end = Endpoints::new(thread_tx, controller_rx);
        let thread_end = Endpoints::new(controller_tx, thread_rx);
        //Not a big deal, but want to drop thread name at least, so no clone
        self.registry.insert(identifier, controller_end);
        thread_end
    }

    fn wait_on_inbox(&self) -> Result<Message> {
        Ok(self.mailbox.receive_blocking()?)
    }

    fn send_to_thread(&self, identifier: Identifier, message: Message) -> Result<()> {
        match self.registry.get(&identifier) {
            Some(value) => value.send(message)?,
            None => bail!("Thread does not exist in registry"),
        };
        Ok(())
    }
}

pub fn thread(config_file: String) -> Result<()> {
    let current_config = Config::new(config_file)?;

    info!("Connecting using specified configuration");
    let mut opened_connection = open_connection(current_config.connection)?;

    let mut hub = Controller::new();
    let handler_endpoint = hub.add_link(Identifier::Handler);
    let runner_endpoint = hub.add_link(Identifier::Runner);

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

// fn process_message(hub: Controller) -> Result<()> {
//     loop {
//         let message = hub.wait_on_inbox()?;
//         match message.origin.as_str() {
//             "handler" => {}
//             "runner" => {}
//             _ => {}
//         };
//     }
// }

// fn process_handler_message(endpoints: &Endpoints, event: Event) {
//     match event {}
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel;
    use pretty_assertions::assert_eq;

    fn setup() -> Endpoints {
        let (tx, rx) = channel::unbounded();
        Endpoints::new( tx, rx)
    }

    #[test]
    fn send_all_multiple_pass() {
        let channels = setup();
        let messages = vec![Message::StopRunning, Message::StopRunning];

        channels
            .send_all(messages)
            .expect("Failed to send multiple messages");
        let messages = channels
            .try_receive_all()
            .expect("Failed to receive multiple messages");
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn try_receive_all_single_pass() {
        let channels = setup();
        let messages = Message::StopRunning;

        channels
            .send(messages)
            .expect("Failed to send single message");
        let message = channels
            .try_receive_all()
            .expect("Failed to receive single message");
        assert_eq!(message.len(), 1);
    }

    #[test]
    fn try_receive_all_multiple_pass() {
        let channels = setup();
        let messages = vec![Message::StopRunning, Message::StopRunning];

        channels
            .send_all(messages)
            .expect("Failed to send multiple messages");
        let messages = channels
            .try_receive_all()
            .expect("Failed to receive multiple messages");
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn try_receive_all_nothing_pass() {
        let channels = setup();
        let result = channels.try_receive_all();

        assert!(result.is_ok(), "Failed to fail at receiving data");
    }

    #[test]
    fn receive_timeout_timed_out_fail() {
        let channels = setup();
        let result = channels.receive_timeout(Duration::from_secs(2));

        assert!(result.is_err(), "Failed to timeout");
    }

    #[test]
    fn receive_timeout_pass() {
        let channels = setup();
        channels
            .send(Message::StopRunning)
            .expect("Failed to send message");

        let result = channels.receive_timeout(Duration::from_secs(2));

        assert!(result.is_ok(), "Failed to get message");
    }

    #[test]
    fn send_pass() {
        let channels = setup();
        channels
            .send(Message::StopRunning)
            .expect("Failed to send message");

        let result = channels
            .receive_timeout(Duration::from_secs(2))
            .expect("Failed to receive message");

        assert_eq!(
            result,
            Message::StopRunning
        );
    }
}
