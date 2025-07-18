use anyhow::{Ok, Result};
use std::io::{Read, Write};

pub trait Communicate: Read + Write {
    fn read_until(&mut self, buf: &mut [u8], until: u8) -> Result<usize> {
        let mut bytes_read = 0;
        for i in 0..buf.len() {
            let result = self.read(&mut buf[i..i + 1])?;
            if result == 0 {
                break;
            }
            bytes_read += 1;
            if buf[i] == until {
                break;
            }
        }

        Ok(bytes_read)
    }
}
