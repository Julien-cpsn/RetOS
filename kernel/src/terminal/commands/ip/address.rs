use crate::add_verbosity;
use crate::devices::network::interface::NETWORK_INTERFACES;
use crate::terminal::arguments::ip_address::{IpCidrArg};
use crate::terminal::arguments::network_interface::NetworkInterfaceArg;
use crate::terminal::error::CliError;
use alloc::format;
use embedded_cli::Command;
use goolog::{debug, info, trace};
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
        },

        /// Delete an IP address from an interface
        Delete {
            /// IP address to delete from the interface
            address: IpCidrArg,

            /// Interface to delete the address from
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

    info!("Adding IP address");
    iface.update_ip_addrs(|addrs| {
        addrs.push(ip_address).unwrap();
    });

    trace!("NETWORK_INTERFACES mutex freed");

    Ok(())
}

pub fn ip_address_delete(ip_address: IpCidr, interface_name: &str) -> Result<(), CliError> {
    trace!("IP ADDRESS DELETE");

    trace!("Locking NETWORK_INTERFACES mutex...");
    let mut network_interfaces = NETWORK_INTERFACES.write();

    trace!("Retrieving network interface \"{}\"", interface_name);
    let iface = network_interfaces.get_mut(interface_name).unwrap();

    debug!("Finding IP address");
    let mut was_address_found = false;

    iface.update_ip_addrs(|addresses| {
        addresses.retain(|address| {
            if address == &ip_address {
                info!("Deleting IP address");
                was_address_found = true;
                false
            }
            else {
                true
            }
        })
    });

    trace!("NETWORK_INTERFACES mutex freed");

    if was_address_found {
        Ok(())
    }
    else {
        Err(CliError::Message(format!("Address \"{}\" not found in interface \"{}\"", ip_address, interface_name)))
    }
}