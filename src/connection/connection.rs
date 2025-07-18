use anyhow::Result;

pub trait Communicate {
    fn receive(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn send(&mut self, buf: &[u8]) -> Result<()>;
}
