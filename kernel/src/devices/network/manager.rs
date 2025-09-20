use crate::devices::network::controller::NetworkController;
use crate::devices::network::driver::NetworkDriver;
use crate::devices::network::interface::{format_mac, init_loopback_interface, init_network_device_interface};
use crate::devices::pic::pic::PIC;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::{format, vec};
use alloc::string::String;
use alloc::vec::Vec;
use goolog::{info, trace};
use smoltcp::iface::{Interface, SocketSet};
use spin::{Lazy, Mutex};
use crate::devices::network::device::NetworkDevice;

const GOOLOG_TARGET: &str = "NETWORK";

pub const NETWORK_DEVICES_INTERRUPT_IRQ: u8 = 0x70;

pub static NETWORK_MANAGER: Lazy<Mutex<NetworkManager>> = Lazy::new(|| Mutex::new(NetworkManager::new()));

pub struct NetworkManager<'a> {
    pub irq_to_devices: BTreeMap<u8, Vec<String>>,
    pub loopback: Interface,
    pub interfaces: BTreeMap<String, Arc<Mutex<NetworkDevice<'a>>>>
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
            sockets: Arc::new(Mutex::new(SocketSet::new(vec![]))),
        };

        let device_index = self.interfaces.len();
        let number_lines = self.irq_to_devices.len();

        self.interfaces.insert(name.clone(), Arc::new(Mutex::new(device)));

        // Map interrupt line to interface id
        self.irq_to_devices.entry(interrupt_line).or_default().push(name);

        trace!("Interface: eth{}, line: 0x{:X}", device_index, interrupt_line);

        if self.irq_to_devices.get(&interrupt_line).unwrap().len() == 1 {
            let shared_vector = NETWORK_DEVICES_INTERRUPT_IRQ + number_lines as u8;
            trace!("Registered interrupt vector: 0x{:X}", shared_vector);

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

        trace!("Handling network interrupt");

        for device_index in devices {
            // iterate all devices that use this vector
            let Some(device) = self.interfaces.get_mut(device_index) else {
                continue;
            };

            // check the NIC's ISR/status register and clear its interrupt sources.
            device.lock().network_controller.process_interrupt();
        }
    }

    pub fn poll_interfaces(&mut self) {
        for device in self.interfaces.values_mut() {
            // let smoltcp process the packets the driver delivered
            if let Some(mut locked_device) = device.try_lock() {
                locked_device.poll();
            }
        }
    }
}
