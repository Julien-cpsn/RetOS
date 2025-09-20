use crate::devices::network::manager::NETWORK_MANAGER;
use crate::terminal::custom_arguments::ip_address::IpCidrArg;
use crate::terminal::custom_arguments::network_interface::NetworkInterfaceArg;
use crate::terminal::error::CliError;
use alloc::format;
use goolog::{debug, info, trace};
use no_std_clap_macros::{Args, Subcommand};
use smoltcp::wire::{IpCidr};

const GOOLOG_TARGET: &str = "IP ADDRESS";

#[derive(Subcommand)]
pub enum IpAddressCommand {
    /// Add an IP address to an interface
    Add(IpAddressAddCommand),

    /// Delete an IP address from an interface
    Delete(IpAddressDeleteCommand),
}

#[derive(Args)]
pub struct IpAddressAddCommand {
    /// IP address to add to the interface
    pub address: IpCidrArg,

    /// Interface to add the address to
    pub interface_name: NetworkInterfaceArg,
}

#[derive(Args)]
pub struct IpAddressDeleteCommand {
    /// IP address to delete from the interface
    pub address: IpCidrArg,

    /// Interface to delete the address from
    pub interface_name: NetworkInterfaceArg,
}

pub fn ip_address_add(ip_address: IpCidr, interface_name: &str) -> Result<(), CliError> {
    trace!("IP ADDRESS ADD");

    trace!("Locking NETWORK_INTERFACES mutex...");
    let mut network_manager = NETWORK_MANAGER.lock();

    trace!("Retrieving network interface \"{}\"", interface_name);
    let device = network_manager.interfaces.get_mut(interface_name).unwrap();
    let mut locked_device = device.lock();
    let iface = &mut locked_device.interface;

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

    let mut network_manager = NETWORK_MANAGER.lock();

    trace!("Retrieving network interface \"{}\"", interface_name);
    let device = network_manager.interfaces.get_mut(interface_name).unwrap();
    let mut locked_device = device.lock();
    let iface = &mut locked_device.interface;

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