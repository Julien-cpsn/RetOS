use crate::terminal::commands::ip::address::IpAddressCommand;
use crate::terminal::commands::ip::interface::IpInterfaceCommand;
use crate::terminal::commands::ip::route::IpRouteCommand;
use no_std_clap_macros::Subcommand;

#[derive(Subcommand)]
pub enum IpCommand {
    /// Interact with network interfaces
    #[command(subcommand)]
    Interface(Option<IpInterfaceCommand>),

    /// Interact with network interfaces
    #[command(subcommand)]
    I(Option<IpInterfaceCommand>),

    /// Interact with network addresses
    #[command(subcommand)]
    Address(Option<IpAddressCommand>),

    /// Interact with network addresses
    #[command(subcommand)]
    A(Option<IpAddressCommand>),

    /// Interact with network routes
    #[command(subcommand)]
    Route(Option<IpRouteCommand>),

    /// Interact with network routes
    #[command(subcommand)]
    R(Option<IpRouteCommand>),
}