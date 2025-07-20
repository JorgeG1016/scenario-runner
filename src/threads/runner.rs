use crate::connection::Communicate;


pub fn thread(connection_handle: Box<dyn Communicate + Send + 'static>){}