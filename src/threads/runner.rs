use crate::{connection::Communicate, threads::Itc};
use log::info;

pub fn thread(connection_handle: &mut Box<dyn Communicate + Send + 'static>, channels: Itc) {
    info!("Starting Command Runner Thread");
    let mut buf: [u8; 100] = [0; 100];
    let _result = connection_handle.read_until(&mut buf, b'\n');
    info!("{buf:?}");
}
