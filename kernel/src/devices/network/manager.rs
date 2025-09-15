use crate::clock::Clock;
use crate::devices::network::controller::NetworkController;
use crate::devices::network::driver::NetworkDriver;
use crate::devices::network::interface::{format_mac, init_loopback_interface, init_network_device_interface};
use crate::devices::pic::pic::PIC;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::{format, vec};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use goolog::info;
use smoltcp::iface::{Interface, SocketSet};
use spin::{Lazy, Mutex};
use crate::devices::network::device::NetworkDevice;
use crate::println;

const GOOLOG_TARGET: &str = "NETWORK";

pub const NETWORK_DEVICES_INTERRUPT_IRQ: u8 = 0x70;

pub static NETWORK_MANAGER: Lazy<Mutex<NetworkManager>> = Lazy::new(|| Mutex::new(NetworkManager::new()));

pub struct NetworkManager<'a> {
    pub irq_to_devices: BTreeMap<u8, Vec<String>>,
    pub loopback: Interface,
    pub interfaces: BTreeMap<String, NetworkDevice<'a>>
}

impl Default for NetworkManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManager<'_> {
    pub fn new() -> Self {
        NetworkManager {
            irq_to_devices: BTreeMap::new(),
            loopback: init_loopback_interface(),
            interfaces: BTreeMap::new(),
        }
    }
    
    pub fn register_device(&mut self, interrupt_line: u8, network_driver: Arc<Mutex<dyn NetworkDriver>>) {
        {
            let driver = network_driver.lock();
            info!("Device detected: {}", driver.nic_type());
            info!("MAC address: {}", format_mac(&driver.mac()));
        }
        
        let mut network_controller = NetworkController::new(network_driver);

        let name = format!("eth{}", self.interfaces.len());
        let interface = init_network_device_interface(&mut network_controller);
        
        let device = NetworkDevice {
            interface,
            network_controller,
            sockets: RefCell::new(SocketSet::new(vec![])),
        };

        let device_index = self.interfaces.len();
        let number_lines = self.irq_to_devices.len();

        self.interfaces.insert(name.clone(), device);

        // Map interrupt line to interface id
        self.irq_to_devices.entry(interrupt_line).or_default().push(name);

        println!("Index: {}, line: 0x{:X}", device_index, interrupt_line);

        if self.irq_to_devices.get(&interrupt_line).unwrap().len() == 1 {
            let shared_vector = NETWORK_DEVICES_INTERRUPT_IRQ + number_lines as u8;
            println!("Registered vector: 0x{:X}", shared_vector);

            PIC
                .get()
                .unwrap()
                .lock()
                .register_interrupt(interrupt_line, shared_vector);
        }
    }

    pub fn handle_interrupt(&mut self, interrupt_line: u8) {
        let Some(devices) = self.irq_to_devices.get(&interrupt_line) else {
            return;
        };

        for device_index in devices {
            // iterate all devices that use this vector
            let Some(device) = self.interfaces.get_mut(device_index) else {
                continue;
            };

            // check the NIC's ISR/status register and clear its interrupt sources.
            if !device.network_controller.process_interrupt() {
                continue;
            }

            // let smoltcp process the packets the driver delivered
            let timestamp = Clock::now();
            device.interface.poll(
                timestamp,
                &mut device.network_controller,
                &mut device.sockets.borrow_mut()
            );
        }
    }
}
