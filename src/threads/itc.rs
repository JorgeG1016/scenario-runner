use chrono::{DateTime, Local};
use std::sync::mpsc::{Receiver, Sender};

pub enum Messages {
    SendData {
        data: Vec<u8>,
    },
    DataReceived {
        timestamp: DateTime<Local>,
        data: Vec<u8>,
    },
    StopRunning,
}

pub struct Itc {
    pub send_channel: Sender<Messages>,
    pub receive_channel: Receiver<Messages>,
}

impl Itc {
    pub fn new(send_channel: Sender<Messages>, receive_channel: Receiver<Messages>) -> Self {
        Self {
            send_channel,
            receive_channel,
        }
    }
}
