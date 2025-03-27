use crate::add_verbosity;
use crate::devices::network_controller::NETWORK_INTERFACES;
use crate::terminal::arguments::ip_address::{IpCidrArg};
use crate::terminal::arguments::network_interface::NetworkInterfaceArg;
use crate::terminal::error::CliError;
use embedded_cli::Command;
use goolog::trace;
use smoltcp::wire::{IpCidr};

const GOOLOG_TARGET: &str = "IP ADDRESS";

add_verbosity! {
    #[derive(Command)]
    pub enum IpAddressCommand<'a> {
        /// Add an IP address to an interface
        Add {
            /// IP address to add to the interface
            address: IpCidrArg,

            /// Interface to add the address to
            interface_name: NetworkInterfaceArg<'a>,
        }
    }
}

pub fn ip_address_add(ip_address: IpCidr, interface_name: &str) -> Result<(), CliError> {
    trace!("IP ADDRESS ADD");

    trace!("Locking NETWORK_INTERFACES mutex...");
    let mut network_interfaces = NETWORK_INTERFACES.write();

    trace!("Retrieving network interface \"{}\"", interface_name);
    let iface = network_interfaces.get_mut(interface_name).unwrap();

    trace!("Adding IP address");
    iface.update_ip_addrs(|addrs| {
        addrs.push(ip_address).unwrap();
    });
    trace!("NETWORK_INTERFACES mutex freed");

    Ok(())
}