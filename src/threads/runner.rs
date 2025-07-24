use super::itc::{Itc, Message};
use crate::connection::Communicate;
use chrono::Local;
use log::{info, trace, warn};

pub fn thread(connection_handle: &mut Box<dyn Communicate + Send + 'static>, channels: Itc) {
    info!("Starting Command Runner Thread!");

    let mut data_stream_enabled = false;
    let mut alive = true;
    while alive {
        if let Ok(messages) = channels.try_receive_all() {
            for message in messages {
                match message {
                    Message::StartDataStream => data_stream_enabled = true,
                    Message::StopDataStream => data_stream_enabled = false,
                    Message::StopRunning => alive = false,
                    Message::SendData { data } => {
                        trace!("Sending data on connection");
                        if connection_handle.write(&data).is_err() {
                            let _ = channels.send(Message::SendError);
                        }
                    }
                    _ => {
                        warn!("Unexpected message received");
                    }
                }
            }
        }

        let mut buf: [u8; 256] = [0; 256];
        if let Ok(data_length) = connection_handle.read_until(&mut buf, b'\n') {
            if data_stream_enabled {
                let _ = channels.send(Message::DataReceived {
                    timestamp: Local::now(),
                    data: Vec::from(buf),
                    data_length,
                });
            }
        }
    }
    info!("Command Runner thread has stopped!");
}
