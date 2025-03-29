use crate::devices::drivers::e1000::E1000;
use crate::devices::drivers::rtl8139::RTL8139;
use crate::interrupts::idt::{network_packet_handler, register_interrupt};
use crate::interrupts::interrupt::InterruptIndex;
use alloc::boxed::Box;
use goolog::{info};
use spin::{Once};
use strum::Display;
use crate::devices::network::interface::{format_mac, init_network_interfaces};

const GOOLOG_TARGET: &str = "NETWORK";

pub static NETWORK_CONTROLLER: Once<NetworkController> = Once::new();

#[derive(Debug, Display)]
pub enum NetworkController {
    RTL8139(Box<RTL8139>),
    E1000(Box<E1000>),
}

pub fn register_network_controller(network_controller: NetworkController, interrupt_line: u32) {
    info!("Device detected: {network_controller}");
    let mac = network_controller.mac();
    info!("MAC address: {}", format_mac(&mac));

    NETWORK_CONTROLLER.call_once(|| network_controller);
    register_interrupt(
        0x10 + 2 * interrupt_line,
        InterruptIndex::NetworkPacket,
        network_packet_handler,
    );
    
    init_network_interfaces();
}

impl NetworkController {
    pub fn mac(&self) -> [u8; 6] {
        match self {
            NetworkController::RTL8139(rtl8139) => rtl8139.mac,
            NetworkController::E1000(e1000) => e1000.mac,
        }
    }

    pub fn send_sync(&self, buffer: &[u8]) {
        match self {
            NetworkController::RTL8139(rtl8139) => rtl8139.send_sync(buffer),
            NetworkController::E1000(e1000) => e1000.send_sync(buffer)
        }
    }

    pub fn end_interrupt(&self) {
        match self {
            NetworkController::RTL8139(rtl8139) => rtl8139.on_interrupt(),
            NetworkController::E1000(e1000) => e1000.on_interrupt(),
        }
    }
}