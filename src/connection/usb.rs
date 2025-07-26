use anyhow::{Ok, Result};
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

use super::Communicate;

pub struct Connection(Box<dyn SerialPort>);

impl Connection {
    pub fn new(port: String, baud_rate: u32) -> Result<Self> {
        let new_connection = serialport::new(port, baud_rate)
            .timeout(Duration::from_secs(1))
            .open()?;
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

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {

    use super::*;
    use nix::pty::{OpenptyResult, PtyMaster, openpty, ptsname_r};
    use nix::unistd::{read, write};
    use std::os::fd::AsFd;

    fn setup() -> OpenptyResult {
        openpty(None, None).expect("Failed to open dummy port")
    }

    #[test]
    fn connection_new_port_open_fail() {
        assert!(
            Connection::new("port/that/does/not/exist".to_string(), 115200).is_err(),
            "There are major issues if this port actually exists"
        );
    }

    #[test]
    fn connection_new_pass() {
        let dummy_port = setup();
        let master_fd = dummy_port.master;
        let master_pty = unsafe { PtyMaster::from_owned_fd(master_fd) };
        let dummy_port_path = ptsname_r(&master_pty).expect("Failed to get dummy port path");
        assert!(
            Connection::new(dummy_port_path.clone(), 115200).is_ok(),
            "Failed to open the dummy port"
        );
    }

    #[test]
    fn connection_read_pass() {
        let dummy_port = setup();
        let master_fd = dummy_port.master;
        let master_pty = unsafe { PtyMaster::from_owned_fd(master_fd) };
        let dummy_port_path = ptsname_r(&master_pty).expect("Failed to get dummy port path");
        let mut new_connection =
            Connection::new(dummy_port_path, 115200).expect("Failed to open dummy serial port");

        let message = b"Hello World!";
        write(master_pty.as_fd(), message).expect("Dummy port write failed");

        let mut buf: [u8; 12] = [0; 12];
        new_connection
            .read(&mut buf)
            .expect("Serial port read failed");
        assert_eq!(buf.as_slice(), message);
    }

    #[test]
    fn connection_write_and_flush_pass() {
        let dummy_port = setup();
        let master_fd = dummy_port.master;
        let master_pty = unsafe { PtyMaster::from_owned_fd(master_fd) };
        let dummy_port_path = ptsname_r(&master_pty).expect("Failed to get dummy port path");
        let mut new_connection =
            Connection::new(dummy_port_path, 115200).expect("Failed to open dummy serial port");

        let message = b"Hello World!";
        new_connection
            .write(message)
            .expect("Serial port write failed");
        new_connection.flush().expect("Serial Port flush failed");
        let mut buf: [u8; 12] = [0; 12];
        read(master_pty.as_fd(), &mut buf).expect("Dummy port read failed");

        assert_eq!(buf.as_slice(), message);
    }
}
