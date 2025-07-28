use anyhow::Result;
use chrono::{DateTime, Local};
use crossbeam::channel::{Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub origin: String,
    pub destination: String,
    pub event: Event,
}

#[derive(Debug, Clone)]
pub struct Endpoints {
    source: String,
    send_channel: Sender<Message>,
    receive_channel: Receiver<Message>,
}

impl Endpoints {
    pub fn new(
        source: String,
        send_channel: Sender<Message>,
        receive_channel: Receiver<Message>,
    ) -> Self {
        Self {
            source,
            send_channel,
            receive_channel,
        }
    }

    pub fn send_all(&self, data_to_send: Vec<Event>, destination: String) -> Result<()> {
        for data in data_to_send {
            self.send_channel.send(Message {
                origin: self.source.clone(),
                event: data,
                destination: destination.clone(),
            })?;
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

    pub fn send(&self, data: Event, destination: String) -> Result<()> {
        Ok(self.send_channel.send(Message {
            origin: self.source.clone(),
            event: data,
            destination,
        })?)
    }

    pub fn get_channels(&self) -> (Sender<Message>, Receiver<Message>) {
        (self.send_channel.clone(), self.receive_channel.clone())
    }

    pub fn get_source(&self) -> String {
        self.source.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel;
    use pretty_assertions::assert_eq;

    fn setup() -> Endpoints {
        let (tx, rx) = channel::unbounded();
        Endpoints::new("unit".to_string(), tx, rx)
    }

    #[test]
    fn send_all_multiple_pass() {
        let channels = setup();
        let data_to_send = vec![Event::StopRunning, Event::StopRunning];

        channels
            .send_all(data_to_send, "unit".to_string())
            .expect("Failed to send multiple messages");
        let messages = channels
            .try_receive_all()
            .expect("Failed to receive multiple messages");
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn try_receive_all_single_pass() {
        let channels = setup();
        let data_to_send = Event::StopRunning;

        channels
            .send(data_to_send, "unit".to_string())
            .expect("Failed to send single message");
        let message = channels
            .try_receive_all()
            .expect("Failed to receive single message");
        assert_eq!(message.len(), 1);
    }

    #[test]
    fn try_receive_all_multiple_pass() {
        let channels = setup();
        let data_to_send = vec![Event::StopRunning, Event::StopRunning];

        channels
            .send_all(data_to_send, "unit".to_string())
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
            .send(Event::StopRunning, "unit".to_string())
            .expect("Failed to send message");

        let result = channels.receive_timeout(Duration::from_secs(2));

        assert!(result.is_ok(), "Failed to get message");
    }

    #[test]
    fn send_pass() {
        let channels = setup();
        channels
            .send(Event::StopRunning, "unit".to_string())
            .expect("Failed to send message");

        let result = channels
            .receive_timeout(Duration::from_secs(2))
            .expect("Failed to receive message");

        assert_eq!(
            result,
            Message {
                origin: channels.source,
                event: Event::StopRunning,
                destination: "unit".to_string()
            }
        );
    }
}
