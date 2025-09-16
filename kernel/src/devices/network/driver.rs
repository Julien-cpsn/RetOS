use alloc::vec::Vec;
use core::fmt::Debug;
use strum::Display;
use crate::devices::drivers::e1000::E1000;
use crate::devices::drivers::rtl8139::RTL8139;

pub trait NetworkDriver: Send + Sync + Debug {
    fn mac(&self) -> [u8; 6];
    fn device_name(&self) -> &str;
    fn nic_type(&self) -> NetworkControllerType;
    fn handle_interrupt(&mut self) -> bool;
    fn send_packet(&mut self, data: &[u8]);
    fn receive_packet(&mut self) -> Option<Vec<u8>>;
}

#[derive(Debug, Display)]
pub enum NetworkControllerType {
    RTL8139,
    E1000
}

impl NetworkDriver for E1000 {
    fn mac(&self) -> [u8; 6] {
        self.mac
    }

    fn device_name(&self) -> &str {
        "E1000"
    }

    fn nic_type(&self) -> NetworkControllerType {
        NetworkControllerType::E1000
    }

    fn handle_interrupt(&mut self) -> bool {
        self.on_interrupt()
    }

    fn send_packet(&mut self, data: &[u8]) {
        self.send_sync(data);
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        self.recv_sync()
    }
}

impl NetworkDriver for RTL8139 {
    fn mac(&self) -> [u8; 6] {
        self.mac
    }

    fn device_name(&self) -> &str {
        "RTL8139"
    }

    fn nic_type(&self) -> NetworkControllerType {
        NetworkControllerType::RTL8139
    }

    fn handle_interrupt(&mut self) -> bool {
        self.on_interrupt()
    }

    fn send_packet(&mut self, data: &[u8]) {
        self.send_sync(data);
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        self.recv_sync()
    }
}