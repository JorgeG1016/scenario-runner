use anyhow::{Ok, Result};
use std::io::{Read, Write};
use std::net::TcpStream;

use super::Communicate;

pub struct Connection(TcpStream);

impl Connection {
    pub fn new(address: String, port: u16) -> Result<Self> {
        let new_connection = TcpStream::connect(format!("{address}:{port}"))?;
        Ok(Connection(new_connection))
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl Communicate for Connection {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn connection_new_pass() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to open tcp listener");
        let listener_addr = listener
            .local_addr()
            .expect("Failed to get test server port and address");
        let listener_ip = listener_addr.ip().to_string();
        let listener_port = listener_addr.port();

        // Need a thread to avoid blocking the thread the test runs in
        let handle = thread::spawn(move || {
            Connection::new(listener_ip, listener_port).expect("Failed to connect")
        });
        listener.accept().expect("Failed to accept connection");
        assert!(handle.join().is_ok(), "Thread joined with panic");
    }

    #[test]
    fn connection_read_and_write_pass() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to open tcp listener");
        let listener_addr = listener
            .local_addr()
            .expect("Failed to get test server port and address");
        let listener_ip = listener_addr.ip().to_string();
        let listener_port = listener_addr.port();

        // Need a thread to avoid blocking the thread the test runs in
        let handle = thread::spawn(move || {
            let mut client =
                Connection::new(listener_ip, listener_port).expect("Failed to connect");
            let mut buf: [u8; 12] = [0; 12];
            let _ = client.read(&mut buf).expect("Failed to read from client");
            let _ = client.write(&buf).expect("Failed to write from client");
            client.flush().expect("Failed to flush data from client");
        });
        let (mut server, _) = listener.accept().expect("Failed to accept connection");

        let sent = b"Hello World!";
        server.write_all(sent).expect("Failed server write");
        let mut received: [u8; 12] = [0; 12];
        // Read should work even after client disconnects, data will have been delivered by then
        let _ = server.read(&mut received).expect("Failed server read");

        assert!(handle.join().is_ok(), "Thread joined with panic");
        assert_eq!(sent, received.as_slice());
    }
}
