use crate::devices::network::driver::NetworkDriver;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use spin::Mutex;

#[derive(Debug)]
pub struct NetworkController {
    pub driver: Arc<Mutex<dyn NetworkDriver>>,
    pub rx_buffer: RefCell<Option<Vec<u8>>>,
    pub capabilities: DeviceCapabilities
}

impl NetworkController {
    pub fn new(driver: Arc<Mutex<dyn NetworkDriver>>) -> NetworkController {
        let mut capabilities = DeviceCapabilities::default();
        capabilities.medium = Medium::Ethernet;
        capabilities.max_transmission_unit = 1500;

        Self {
            driver,
            rx_buffer: RefCell::new(None),
            capabilities
        }
    }

    pub fn process_interrupt(&self) -> bool {
        let mut network_driver = self.driver.lock();

        if network_driver.handle_interrupt() {
            if let Some(packet) = network_driver.receive_packet() {
                *self.rx_buffer.borrow_mut() = Some(packet);
            }
        }

        return true;
    }
}

impl Device for NetworkController {
    type RxToken<'a> = PhyRxToken<'a> where Self: 'a;
    type TxToken<'a> = PhyTxToken<'a> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        //println!("recv");

        if self.rx_buffer.borrow().is_some() {
            Some((
                PhyRxToken { device: self },
                PhyTxToken { device: self }
            ))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        //println!("send");

        Some(PhyTxToken { device: self })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        self.capabilities.clone()
    }
}

pub struct PhyRxToken<'a> {
    device: &'a NetworkController,
}
pub struct PhyTxToken<'a> {
    device: &'a NetworkController,
}

impl<'a> RxToken for PhyRxToken<'a> {
    fn consume<R, F>(self, f: F) -> R where F: FnOnce(&[u8]) -> R {
        //println!("consume rx");

        let mut buffer = self.device.rx_buffer.borrow_mut();
        if let Some(packet) = buffer.take() {
            f(&packet)
        } else {
            // This shouldn't happen if we properly handle interrupts
            panic!("RxToken consumed without available packet");
        }
    }
}

impl<'a> TxToken for PhyTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R where F: FnOnce(&mut [u8]) -> R {
        //println!("consume tx");
        // Allocate a buffer for the packet
        let mut buffer = vec![0u8; len];

        // Call the function to fill the buffer
        let result = f(&mut buffer);

        // Send the packet
        self.device.driver.lock().send_packet(&buffer);

        result
    }
}