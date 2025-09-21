use crate::printer::buffer::WRITER;
use crate::terminal::error::CliError;
use crate::devices::network::manager::NETWORK_MANAGER;
use crate::terminal::custom_arguments::ip_address::{IpAddressArg, IpCidrArg};
use crate::terminal::custom_arguments::network_interface::NetworkInterfaceArg;
use alloc::string::{String, ToString};
use alloc::{format, vec};
use goolog::{debug, info, trace};
use no_std_clap_macros::{Args, Subcommand};
use smoltcp::iface::Route;
use smoltcp::wire::{IpAddress, IpCidr};

const GOOLOG_TARGET: &str = "IP ROUTE";

#[derive(Subcommand)]
pub enum IpRouteCommand {
    /// Show network routes
    Show,

    /// Add an IP route to an interface
    Add(IpRouteAddCommand),

    /// Delete an IP route from an interface
    Delete(IpRouteDeleteCommand)
}

#[derive(Args)]
pub struct IpRouteAddCommand {
    /// IP with Cidr route to add to the interface
    pub address: IpCidrArg,

    /// Interface to add the route to
    pub interface_name: NetworkInterfaceArg,

    /// IP gateway. Defaults to: 0.0.0.0
    #[arg(default_value = "0.0.0.0")]
    pub gateway: IpAddressArg,
}

#[derive(Args)]
pub struct IpRouteDeleteCommand {
    /// IP with Cidr route to delete from the interface
    pub address: IpCidrArg,

    /// Interface to delete the route from
    pub interface_name: NetworkInterfaceArg,
}

pub fn ip_route_show() -> Result<(), CliError> {
    trace!("IP ROUTE SHOW");

    let mut table = vec![
        [String::from("Interface"), String::from("IP"), String::from("Gateway"), String::from("Expires at"), String::from("Preferred until")]
    ];

    trace!("Locking NETWORK_INTERFACES mutex...");
    let mut network_manager = NETWORK_MANAGER.lock();

    for (name, device) in network_manager.interfaces.iter_mut() {
        let mut locked_device = device.lock();

        locked_device.interface
            .routes_mut()
            .update(|route_list| {
                for route in route_list.iter() {
                    let expires_at = match route.expires_at {
                        None => String::new(),
                        Some(instant) => instant.to_string()
                    };

                    let preferred_until = match route.preferred_until {
                        None => String::new(),
                        Some(instant) => instant.to_string()
                    };

                    table.push([name.to_string(), route.cidr.to_string(), route.via_router.to_string(), expires_at, preferred_until]);
                }
            });
    }
    trace!("NETWORK_INTERFACES mutex freed");

    let mut writer = WRITER.write();
    text_tables::render(&mut *writer, table).unwrap();

    Ok(())
}

pub fn ip_route_add(ip_address: IpCidr, interface_name: &str, gateway: IpAddress) -> Result<(), CliError> {
    trace!("IP ROUTE ADD");

    trace!("Locking NETWORK_INTERFACES mutex...");
    let mut network_manager = NETWORK_MANAGER.lock();

    trace!("Retrieving network interface \"{}\"", interface_name);
    let device = network_manager.interfaces.get_mut(interface_name).unwrap();
    let mut locked_device = device.lock();
    let iface = &mut locked_device.interface;

    info!("Adding IP route");
    iface
        .routes_mut()
        .update(|routes| {
        routes
                .push(Route {
                    cidr: ip_address,
                    via_router: gateway,
                    preferred_until: None,
                    expires_at: None,
                })
                .unwrap();
        });

    trace!("NETWORK_INTERFACES mutex freed");

    Ok(())
}

pub fn ip_route_delete(ip_address: IpCidr, interface_name: &str) -> Result<(), CliError> {
    trace!("IP ROUTE DELETE");

    trace!("Locking NETWORK_INTERFACES mutex...");
    let mut network_manager = NETWORK_MANAGER.lock();

    trace!("Retrieving network interface \"{}\"", interface_name);
    let device = network_manager.interfaces.get_mut(interface_name).unwrap();
    let mut locked_device = device.lock();
    let iface = &mut locked_device.interface;

    debug!("Finding IP route");
    let mut was_route_found = false;
    
    iface
        .routes_mut()
        .update(|routes| {
            routes.retain(|route| {
                if route.cidr == ip_address {
                    info!("Deleting IP route");
                    was_route_found = true;
                    false
                }
                else {
                    true
                }
            })
        });

    trace!("NETWORK_INTERFACES mutex freed");

    if was_route_found {
        Ok(())
    }
    else {
        Err(CliError::Message(format!("Route \"{}\" not found in interface \"{}\"", ip_address, interface_name)))
    }
}