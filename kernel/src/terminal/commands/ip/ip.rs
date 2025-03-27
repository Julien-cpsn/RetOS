use alloc::string::{String, ToString};
use alloc::vec;
use embedded_cli::Command;
use crate::terminal::error::CliError;
use goolog::{trace};
use crate::add_verbosity;
use crate::devices::network_controller::{format_mac, NETWORK_INTERFACES};
use crate::printer::buffer::WRITER;

const GOOLOG_TARGET: &str = "IP";

add_verbosity! {
    #[derive(Command)]
    pub enum IpCommand {
        /// Show network interfaces
        Interface
    }
}

pub fn ip_show() -> Result<(), CliError> {
    trace!("IP SHOW");

    let mut table = vec![
        [String::from("Interface"), String::from("MAC address"), String::from("IPv4"), String::from("IPv6")]
    ];
    
    let network_interfaces = NETWORK_INTERFACES.read();

    for (name, interface) in network_interfaces.iter() {
        let mac = interface.hardware_addr();
        
        let ipv4 = match interface.ipv4_addr() {
            None => String::from("None"),
            Some(ipv4) => ipv4.to_string()
        };
        
        let ipv6 = match interface.ipv6_addr() {
            None => String::from("None"),
            Some(ipv6) => ipv6.to_string()
        };
        
        table.push([name.to_string(), format_mac(mac.as_bytes()), ipv4, ipv6]);
    }

    let mut writer = WRITER.write();
    text_tables::render(&mut *writer, table).unwrap();
    
    Ok(())
}