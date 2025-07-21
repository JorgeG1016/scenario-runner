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
