use anyhow::Result;

pub trait Communicate {
    fn receive_response(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write_command(&mut self, buf: &[u8]) -> Result<usize>;
    fn close() -> Result<()>;
}
