use super::itc::{Itc, Message};
use crate::connection::Communicate;
use chrono::{Date, Local};
use clap::error;
use log::{info, warn, error};

pub fn thread(connection_handle: &mut Box<dyn Communicate + Send + 'static>, channels: Itc) {
    info!("Starting Command Runner Thread");

    let mut data_stream_enabled = false;
    let mut alive = true;
    while alive {
        if let Ok(messages) = channels.try_receive_all() {
            for message in messages {
                match message {
                    Message::StartDataStream => data_stream_enabled = true,
                    Message::StopDataStream => data_stream_enabled= false,
                    Message::StopRunning => alive = false,
                    Message::SendData { data } => {
                        info!("Sending data on connection");
                        if let Err(_) = connection_handle.write(&data) {
                            error!("Couldn't write bytes, check connection");
                        }
                    },
                    _ => {warn!("Unknown message received");}
                }
            }
        }
        
        let mut buf: [u8; 256] = [0; 256];
        if let Ok(data_length) = connection_handle.read_until(&mut buf, b'\n') {
            let buf_str = String::from_utf8_lossy(&buf);
            info!("{}", buf_str);
            if data_stream_enabled {
                channels.send(Message::DataReceived { timestamp: Local::now(), data: Vec::from(buf), data_length: data_length });
            }
        }
    }
}
