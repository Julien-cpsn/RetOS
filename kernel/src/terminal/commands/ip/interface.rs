use crate::add_verbosity;
use crate::printer::buffer::WRITER;
use crate::terminal::error::CliError;
use alloc::string::{String, ToString};
use alloc::vec;
use embedded_cli::Command;
use goolog::trace;
use smoltcp::wire::IpCidr;
use crate::devices::network::interface::{format_mac, NETWORK_INTERFACES};

const GOOLOG_TARGET: &str = "IP INTERFACE";

add_verbosity! {
    #[derive(Command)]
    pub enum IpInterfaceCommand {
        /// Show network interfaces
        Show
    }
}

pub fn ip_interface_show() -> Result<(), CliError> {
    trace!("IP INTERFACE SHOW");

    let mut table = vec![
        [String::from("Interface"), String::from("MAC address"), String::from("IPv4 addresses"), String::from("IPv6 addresses")]
    ];

    let network_interfaces = NETWORK_INTERFACES.read();

    for (name, interface) in network_interfaces.iter() {
        let mac = interface.hardware_addr();

        let mut ips_v4 = vec![];
        let mut ips_v6 = vec![];
        
        for ip in interface.ip_addrs() {
            match ip {
                IpCidr::Ipv4(ipv4) => ips_v4.push(ipv4.to_string()),
                IpCidr::Ipv6(ipv6) => ips_v6.push(ipv6.to_string()),
            }
        }

        table.push([name.to_string(), format_mac(mac.as_bytes()), ips_v4.join(", "), ips_v6.join(", ")]);
    }

    let mut writer = WRITER.write();
    text_tables::render(&mut *writer, table).unwrap();

    Ok(())
}