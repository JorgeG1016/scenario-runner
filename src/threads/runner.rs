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
        if let Ok(bytes_read) = connection_handle.read_until(&mut buf, b'\n') {
            if data_stream_enabled {
                let mut data = Vec::from(buf);
                data.truncate(bytes_read - 1);
                let data_length = data.len();
                let _ = channels.send(Message::DataReceived {
                    timestamp: Local::now(),
                    data,
                    data_length,
                });
            }
        }
    }
    info!("Command Runner thread has stopped!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Communicate;
    use crate::Itc;
    use pretty_assertions::assert_eq;
    use std::io::Read;
    use std::io::Write;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    struct MockConnection {
        message_read: Vec<u8>,
        message_written: Vec<u8>,
        read_index: usize,
    }

    impl Read for MockConnection {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            for byte in buf.iter_mut() {
                if self.read_index >= self.message_read.len() {
                    self.read_index = 0;
                }
                *byte = self.message_read[self.read_index];
                self.read_index += 1;
            }
            Ok(buf.len())
        }
    }

    impl Write for MockConnection {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            for byte in buf {
                self.message_written.push(*byte);
            }
            Ok(self.message_written.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.message_written.clear();
            Ok(())
        }
    }

    impl Communicate for MockConnection {}

    fn setup() -> (MockConnection, Itc, Itc) {
        let (test_tx, test_rx) = channel();
        let (thread_tx, thread_rx) = channel();
        (
            MockConnection {
                message_read: Vec::new(),
                message_written: Vec::new(),
                read_index: 0,
            },
            Itc::new(test_tx, thread_rx),
            Itc::new(thread_tx, test_rx),
        )
    }

    #[test]
    fn thread_stop() {
        let (mut mock_connection, unit_channel, thread_channel) = setup();
        let read_string = "Hello World!\n";
        mock_connection
            .message_read
            .extend_from_slice(read_string.as_bytes());
        let mut mock_connection: Box<dyn Communicate + Send + 'static> = Box::new(mock_connection);

        let handle = thread::spawn(move || thread(&mut mock_connection, thread_channel));
        unit_channel
            .send(Message::StopRunning)
            .expect("Failed to send stop running message");
        assert!(handle.join().is_ok(), "Thread stopped with error thread")
    }

    #[test]
    fn thread_data_receive() {
        let (mut mock_connection, unit_channel, thread_channel) = setup();
        let read_string = "Hello World!\n";
        mock_connection
            .message_read
            .extend_from_slice(read_string.as_bytes());
        let mut mock_connection: Box<dyn Communicate + Send + 'static> = Box::new(mock_connection);

        let handle = thread::spawn(move || thread(&mut mock_connection, thread_channel));
        unit_channel
            .send(Message::StartDataStream)
            .expect("Failed to send start data stream message");

        //Should receive something back way faster than 60 seconds
        let received_message = unit_channel
            .receive_timeout(Duration::from_secs(5))
            .expect("Somehow didn't receive anything back");
        let received_data = match received_message {
            Message::DataReceived { data, .. } => data,
            _ => vec![],
        };
        let received_string =
            String::from_utf8(received_data).expect("Failed to convert bytes to string");

        //String from thread shouldn't contain the end character
        assert_eq!(received_string, read_string.trim_end_matches('\n'));
        unit_channel
            .send(Message::StopRunning)
            .expect("Failed to send stop running message");
        assert!(handle.join().is_ok(), "Thread stopped with error thread")
    }

    #[test]
    fn thread_data_send() {
        let (mut mock_connection, unit_channel, thread_channel) = setup();
        let read_string = "Hello World!\n";
        mock_connection
            .message_read
            .extend_from_slice(read_string.as_bytes());
        let mut mock_connection: Box<dyn Communicate + Send + 'static> = Box::new(mock_connection);

        let handle = thread::spawn(move || thread(&mut mock_connection, thread_channel));
        unit_channel
            .send(Message::SendData {
                data: Vec::from("Hello World!"),
            })
            .expect("Failed to send send data message");
        unit_channel
            .send(Message::StopRunning)
            .expect("Failed to send stop running message");
        assert!(handle.join().is_ok(), "Thread stopped with error thread")
    }
}
