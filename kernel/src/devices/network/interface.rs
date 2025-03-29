use crate::clock::Clock;
use crate::devices::network::device::NetworkDevice;
use crate::devices::network_controller::{NETWORK_CONTROLLER};
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use smoltcp::iface::{Config, Interface};
use smoltcp::phy::{Loopback, Medium};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};
use spin::RwLock;

pub static NETWORK_INTERFACES: RwLock<BTreeMap<String, Interface>> = RwLock::new(BTreeMap::new());

pub fn init_network_interfaces() {
    let mut network_interfaces = NETWORK_INTERFACES.write();
    network_interfaces.clear();

    network_interfaces.insert(String::from("lo"), init_loopback_interface());
    network_interfaces.insert(String::from("eth"), init_network_device_interface());
}

fn init_loopback_interface() -> Interface {
    let mut loopback = Loopback::new(Medium::Ethernet);
    let config = Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into());
    let mut iface = Interface::new(config, &mut loopback, Clock::now());
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs.push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 8)).unwrap();
        ip_addrs.push(IpCidr::new(IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1), 128)).unwrap();
    });

    iface
}

fn init_network_device_interface() -> Interface {
    let mut device = NetworkDevice;
    let mac = NETWORK_CONTROLLER.get().unwrap().mac();
    let config = Config::new(EthernetAddress(mac).into());

    Interface::new(config, &mut device, Clock::now())
}

pub fn format_mac(mac: &[u8]) -> String {
    mac.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<String>>().join(":")
}