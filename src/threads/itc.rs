use anyhow::Result;
use chrono::{DateTime, Local};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

#[allow(dead_code)]
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
}

pub struct Itc {
    send_channel: Sender<Message>,
    receive_channel: Receiver<Message>,
}

impl Itc {
    pub fn new(send_channel: Sender<Message>, receive_channel: Receiver<Message>) -> Self {
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

    pub fn send(&self, message: Message) -> Result<()> {
        Ok(self.send_channel.send(message)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::mpsc::channel;

    fn setup() -> Itc {
        let (tx, rx) = channel();
        Itc::new(tx, rx)
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
        let message = Message::StopRunning;

        channels
            .send(message)
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

        assert!(
            matches!(result, Message::StopRunning),
            "Something unexpected received"
        );
    }
}
