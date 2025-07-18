use anyhow::{Ok, Result};
use serialport::SerialPort;
use std::time::Duration;

use crate::connection::Communicate;

pub struct SerialConnection {
    connection: Box<dyn SerialPort>,
}
impl SerialConnection {
    pub fn new(port: String, baud_rate: u32) -> Result<SerialConnection> {
        let opened_serial_port = serialport::new(port, baud_rate)
            .timeout(Duration::from_secs(1))
            .open()?;
        Ok(SerialConnection {
            connection: opened_serial_port,
        })
    }
}
impl Communicate for SerialConnection {
    fn receive(&mut self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.connection.read(buf)?)
    }

    fn send(&mut self, buf: &[u8]) -> Result<()> {
        Ok(self.connection.write_all(buf)?)
    }
}
