use crate::connection::Communicate;
use crate::threads::controller::{Message, ThreadManager};
use chrono::Local;
use log::{error, info, trace, warn};

pub fn thread(
    connection_handle: &mut Box<dyn Communicate + Send + 'static>,
    manager: ThreadManager,
) {
    info!("Starting Command Runner Thread!");

    'main: loop {
        if let Ok(messages) = manager.try_receive_all() {
            for message in messages {
                match message {
                    Message::StopRunning => break 'main,
                    Message::RunnerSendData { data } => {
                        trace!("Sending data on connection");
                        if connection_handle.write(&data).is_err() {
                            error!("Failed to send bytes");
                            let _ = manager.send(Message::SendError);
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
            let mut data = Vec::from(buf);
            data.truncate(bytes_read - 1);
            let data_length = data.len();
            let _ = manager.send(Message::RunnerReceivedData {
                timestamp: Local::now(),
                data,
                data_length,
            });
        } else {
            error!("Failed to receive bytes");
            let _ = manager.send(Message::ReceiveError);
        }
    }
    info!("Command Runner thread has stopped!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::Communicate;
    use crossbeam::channel;
    use pretty_assertions::assert_eq;
    use std::io::{Error, Read, Write};
    use std::thread;
    use std::time::Duration;

    struct MockConnection {
        message_read: Vec<u8>,
        message_written: Vec<u8>,
        read_index: usize,
    }

    struct FailedReadMockConnection;
    struct FailedWriteMockConnection {
        message_read: Vec<u8>,
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

    impl Read for FailedReadMockConnection {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::other("Simulated read failure"))
        }
    }

    impl Write for FailedReadMockConnection {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Communicate for FailedReadMockConnection {}

    impl Read for FailedWriteMockConnection {
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

    impl Write for FailedWriteMockConnection {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(Error::other("Simulated write failure"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Err(Error::other("Simulated write failure"))
        }
    }

    impl Communicate for FailedWriteMockConnection {}

    fn setup() -> (MockConnection, ThreadManager, ThreadManager) {
        let (test_tx, test_rx) = channel::unbounded();
        let (thread_tx, thread_rx) = channel::unbounded();
        (
            MockConnection {
                message_read: Vec::new(),
                message_written: Vec::new(),
                read_index: 0,
            },
            ThreadManager::new(test_tx, thread_rx),
            ThreadManager::new(thread_tx, test_rx),
        )
    }

    fn fail_read_setup() -> (FailedReadMockConnection, ThreadManager, ThreadManager) {
        let (test_tx, test_rx) = channel::unbounded();
        let (thread_tx, thread_rx) = channel::unbounded();
        (
            FailedReadMockConnection,
            ThreadManager::new(test_tx, thread_rx),
            ThreadManager::new(thread_tx, test_rx),
        )
    }

    fn fail_write_setup() -> (FailedWriteMockConnection, ThreadManager, ThreadManager) {
        let (test_tx, test_rx) = channel::unbounded();
        let (thread_tx, thread_rx) = channel::unbounded();
        (
            FailedWriteMockConnection {
                message_read: Vec::new(),
                read_index: 0,
            },
            ThreadManager::new(test_tx, thread_rx),
            ThreadManager::new(thread_tx, test_rx),
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

        //Should receive something back way faster than 60 seconds
        let received_message = unit_channel
            .receive_timeout(Duration::from_secs(5))
            .expect("Somehow didn't receive anything back");
        let received_data = match received_message {
            Message::RunnerReceivedData { data, .. } => data,
            _ => panic!("Received the wrong data"),
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
            .send(Message::RunnerSendData {
                data: Vec::from("Hello World!"),
            })
            .expect("Failed to send send data message");
        unit_channel
            .send(Message::StopRunning)
            .expect("Failed to send stop running message");

        assert!(handle.join().is_ok(), "Thread stopped with error thread")
    }

    #[test]
    fn thread_data_receive_fail() {
        let (mock_connection, unit_channel, thread_channel) = fail_read_setup();
        let mut mock_connection: Box<dyn Communicate + Send + 'static> = Box::new(mock_connection);
        let handle = thread::spawn(move || thread(&mut mock_connection, thread_channel));

        let received_message = unit_channel
            .receive_timeout(Duration::from_secs(10))
            .expect("Did not receive anything from thread");
        assert!(
            matches!(received_message, Message::ReceiveError),
            "Unexpectedly received something else"
        );

        unit_channel
            .send(Message::StopRunning)
            .expect("Failed to send stop running message");
        assert!(handle.join().is_ok(), "Thread stopped with error thread")
    }

    #[test]
    fn thread_data_send_fail() {
        let (mut mock_connection, unit_channel, thread_channel) = fail_write_setup();
        let read_string = "Hello World!\n";
        mock_connection
            .message_read
            .extend_from_slice(read_string.as_bytes());
        let mut mock_connection: Box<dyn Communicate + Send + 'static> = Box::new(mock_connection);
        let handle = thread::spawn(move || thread(&mut mock_connection, thread_channel));

        unit_channel
            .send(Message::RunnerSendData {
                data: Vec::from("Hello World!"),
            })
            .expect("Failed to send send data message");

        let received_message = unit_channel
            .receive_timeout(Duration::from_secs(10))
            .expect("Did not receive anything from thread");
        assert!(
            matches!(received_message, Message::SendError),
            "Unexpectedly received something else"
        );

        unit_channel
            .send(Message::StopRunning)
            .expect("Failed to send stop running message");
        assert!(handle.join().is_ok(), "Thread stopped with error")
    }

    #[test]
    fn thread_data_send_unhandled_message() {
        let (mut mock_connection, unit_channel, thread_channel) = setup();
        let read_string = "Hello World!\n";
        mock_connection
            .message_read
            .extend_from_slice(read_string.as_bytes());
        let mut mock_connection: Box<dyn Communicate + Send + 'static> = Box::new(mock_connection);

        let handle = thread::spawn(move || thread(&mut mock_connection, thread_channel));
        unit_channel
            .send(Message::SendError)
            .expect("Failed to send unhandled message");
        unit_channel
            .send(Message::StopRunning)
            .expect("Failed to send stop running message");
        assert!(handle.join().is_ok(), "Thread stopped with error thread")
    }
}
