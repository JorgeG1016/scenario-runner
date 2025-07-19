pub mod connection;
pub mod serial;
pub mod tcp;

use connection::Communicate;
use serial::Connection as SerialConnection;
use tcp::Connection as TcpConnection;
