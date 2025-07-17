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
    fn receive_until(&mut self, buf: &mut [u8], until: u8) -> Result<usize> {
        let mut bytes_read: usize = 0;
        for i in 0..buf.len() {
            let n = self.connection.read(&mut buf[i..i + 1])?;
            if n == 0 {
                break;
            }
            bytes_read += 1;
            if buf[i] == until {
                break;
            }
        }
        Ok(bytes_read)
    }

    fn send(&mut self, buf: &[u8]) -> Result<()> {
        self.connection.write_all(buf)?;
        Ok(())
    }
}
