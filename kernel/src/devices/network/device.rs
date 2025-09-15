use crate::devices::network::controller::NetworkController;
use core::cell::RefCell;
use smoltcp::iface::{Interface, SocketSet};


pub struct NetworkDevice<'a> {
    pub interface: Interface,
    pub network_controller: NetworkController,
    pub sockets: RefCell<SocketSet<'a>>
}