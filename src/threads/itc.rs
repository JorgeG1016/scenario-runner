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
