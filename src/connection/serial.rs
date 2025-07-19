use anyhow::{Ok, Result};
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

use crate::connection::Communicate;

pub struct Connection(Box<dyn SerialPort>);

impl Connection {
    fn new(port: String, baud_rate: u32) -> Result<Self> {
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
