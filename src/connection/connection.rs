use anyhow::{Ok, Result};
use std::io::{Read, Write};

#[allow(dead_code)]
pub trait Communicate: Read + Write {
    fn read_until(&mut self, buf: &mut [u8], until: u8) -> Result<usize> {
        let mut bytes_read = 0;
        for i in 0..buf.len() {
            let mut byte: [u8; 1] = [0; 1];
            let result = self.read(&mut byte)?;
            if result == 0 {
                break;
            }
            bytes_read += 1;
            buf[i] = byte[0];
            if buf[i] == until {
                break;
            }
        }

        Ok(bytes_read)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::{Error, ErrorKind, Read, Seek, Write};
    use tempfile::tempfile;

    struct FailedReader;
    impl Read for FailedReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::Other, "Simulated read failure"))
        }
    }

    impl Write for FailedReader {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::Other, "Simulated write failure"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(Error::new(ErrorKind::Other, "Simulated flush failure"))
        }
    }

    impl Communicate for FailedReader {}
    impl Communicate for File {}

    #[test]
    fn read_until_fail_failed_read() {
        let mut failed_reader = FailedReader;
        let mut buf: [u8; 10] = [0; 10];

        let result = failed_reader.read_until(&mut buf, b'\n');

        assert!(result.is_err(), "Somehow this forced failed read passed")
    }

    #[test]
    fn read_until_pass_byte_found() {
        let mut temp_file = tempfile().expect("Failed to create tempfile");
        let mut buf: [u8; 16] = [0; 16];
        writeln!(temp_file, "Hello World!").expect("Failed to write to tempfile");
        temp_file.rewind().expect("Failed to rewind tempfile");

        let bytes_read = temp_file
            .read_until(&mut buf, b'!')
            .expect("Somehow the temp file read failed");

        assert_eq!(bytes_read, String::from("Hello World!").len());
        assert_eq!(
            "Hello World!",
            str::from_utf8(&buf[..bytes_read]).expect("Failed to convert bytes to str")
        );
    }

    #[test]
    fn read_until_pass_byte_not_found() {
        let mut temp_file = tempfile().expect("Failed to create tempfile");
        let mut buf: [u8; 16] = [0; 16];
        writeln!(temp_file, "Hello World!").expect("Failed to write to tempfile");
        temp_file.rewind().expect("Failed to rewind tempfile");

        let bytes_read = temp_file
            .read_until(&mut buf, b'4')
            .expect("Somehow the temp file read failed");

        assert_eq!(bytes_read, String::from("Hello World!\n").len());
        assert_eq!(
            "Hello World!\n",
            str::from_utf8(&buf[..bytes_read]).expect("Failed to convert bytes to str")
        );
    }

    #[test]
    fn read_until_pass_buffer_full() {
        let mut temp_file = tempfile().expect("Failed to create tempfile");
        let mut buf: [u8; 5] = [0; 5];
        writeln!(temp_file, "Hello World!").expect("Failed to write to tempfile");
        temp_file.rewind().expect("Failed to rewind tempfile");

        let bytes_read = temp_file
            .read_until(&mut buf, b'!')
            .expect("Somehow the temp file read failed");

        assert_eq!(bytes_read, String::from("Hello").len());
        assert_eq!(
            "Hello",
            str::from_utf8(&buf[..bytes_read]).expect("Failed to convert bytes to str")
        );
    }
}
