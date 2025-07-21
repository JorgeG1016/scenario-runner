use crate::connection::Communicate;

pub fn thread(connection_handle: &mut Box<dyn Communicate + Send + 'static>) {
    let mut buf: [u8; 100] = [0; 100];
    let result = connection_handle.read_until(&mut buf, b'\n');
    print!("{:?}", buf);
}
