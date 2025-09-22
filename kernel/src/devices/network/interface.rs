use crate::clock::Clock;
use crate::devices::network::controller::NetworkController;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use smoltcp::iface::{Config, Interface};
use smoltcp::phy::{Loopback, Medium};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};

pub fn init_loopback_interface() -> Interface {
    let mut loopback = Loopback::new(Medium::Ethernet);
    let config = Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into());
    let mut iface = Interface::new(config, &mut loopback, Clock::now());
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs.push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 8)).unwrap();
        ip_addrs.push(IpCidr::new(IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1), 128)).unwrap();
    });

    iface
}

pub fn init_network_device_interface(network_controller: &mut NetworkController) -> Interface {
    let driver = network_controller.driver.lock();
    let mac = driver.mac();
    drop(driver);
    let config = Config::new(EthernetAddress(mac).into());

    Interface::new(config, network_controller, Clock::now())
}

pub fn format_mac(mac: &[u8]) -> String {
    mac.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<String>>().join(":")
}