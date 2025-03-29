use crate::add_group_verbosity;
use crate::terminal::commands::ip::address::IpAddressCommand;
use crate::terminal::commands::ip::interface::IpInterfaceCommand;
use embedded_cli::Command;
use crate::terminal::commands::ip::route::IpRouteCommand;

add_group_verbosity! {
    #[derive(Command)]
    pub enum IpCommand<'a> {
        /// Interact with network interfaces
        #[command(subcommand)]
        Interface(IpInterfaceCommand),

        #[command(subcommand)]
        I(IpInterfaceCommand),

        /// Interact with network addresses
        #[command(subcommand)]
        Address(IpAddressCommand<'a>),

        #[command(subcommand)]
        A(IpAddressCommand<'a>),

        /// Interact with network routes
        #[command(subcommand)]
        Route(IpRouteCommand<'a>),

        #[command(subcommand)]
        R(IpRouteCommand<'a>),
    }
}