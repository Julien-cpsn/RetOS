use crate::clock::Clock;
use crate::devices::network::controller::NetworkController;
use alloc::sync::Arc;
use smoltcp::iface::{Interface, SocketSet};
use spin::Mutex;

pub struct NetworkDevice<'a> {
    pub interface: Interface,
    pub network_controller: NetworkController,
    pub sockets: Arc<Mutex<SocketSet<'a>>>
}

impl NetworkDevice<'_> {
    pub fn poll(&mut self) {
        let timestamp = Clock::now();

        if let Some(locked_sockets) = self.sockets.try_lock().as_mut() {
            self.interface.poll(
                timestamp,
                &mut self.network_controller,
                locked_sockets
            );
        }
    }
}