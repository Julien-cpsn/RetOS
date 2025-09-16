use crate::terminal::arguments::ip_address::IpAddressArg;
use crate::terminal::commands::clear::clear;
use crate::terminal::commands::echo::echo;
use crate::terminal::commands::ip::address::{ip_address_add, ip_address_delete, IpAddressCommand};
use crate::terminal::commands::ip::interface::{ip_interface_show, IpInterfaceCommand};
use crate::terminal::commands::ip::ip::IpCommand;
use crate::terminal::commands::ip::route::{ip_route_add, ip_route_delete, ip_route_show, IpRouteCommand};
use crate::terminal::commands::keyboard::{change_layout, KeyboardLayoutArg};
use crate::terminal::commands::lspci::lspci;
use crate::terminal::commands::ping::ping;
use crate::terminal::commands::ps::ps;
use crate::terminal::commands::scanpci::scanpci;
use crate::terminal::commands::shutdown::shutdown;
use crate::terminal::commands::sleep::cli_sleep;
use crate::terminal::commands::top::top;
use crate::terminal::commands::uptime::uptime;
use crate::terminal::error::CliError;
use crate::terminal::terminal::TerminalBuffer;
use crate::add_verbosity;
use alloc::format;
use embedded_cli::cli::Cli;
use embedded_cli::Command;
use goolog::log::{set_max_level, LevelFilter};
use yansi::Paint;

add_verbosity! {
    #[derive(Command)]
    pub enum Command<'a> {
        /// Echoes the following argument
        Echo {
            /// Text to echo
            text: &'a str,
        },

        /// Clear the terminal
        Clear,

        /// List current processes
        Ps,

        /// Print details about system resources usage
        Top,
        
        /// Print for how much time the system is running
        Uptime,

        /// Sleeps the system
        Sleep {
            /// Seconds amount to sleep the system
            seconds: u64,
        },

        /// Shutdown the operating system
        Shutdown,
        
        /// Change the keyboard layout
        Keyboard {
            /// Keyboard layout to use
            layout: KeyboardLayoutArg,
        },
        
        /// List PCI devices
        Lspci,
        
        /// Enforces a PCI device scan
        Scanpci,
        
        /// Ping an IP address
        Ping {
            /// IP address to ping
            ip_address: IpAddressArg,

            /// Ping count
            #[arg(default_value_t = 4)]
            count: u16,

            /// Timeout
            #[arg(default_value_t = 2)]
            timeout: u64
        },
        
        /// Network commands
        Ip {
            #[command(subcommand)]
            subcommand: IpCommand<'a>
        },
    }
}

pub fn handle_command(cli: &mut Cli<&mut TerminalBuffer, CliError, [u8; 100], [u8; 100]>, byte: u8) {
    let mut command_processor = Command::processor(|_cli, command| {
        set_max_verbosity(command.get_verbosity());

        match command {
            Command::Echo { text, .. } => echo(text),
            Command::Clear { .. } => clear(),
            Command::Ps { .. } => ps(),
            Command::Keyboard { layout, .. } => change_layout(layout),
            Command::Lspci { .. } => lspci(),
            Command::Scanpci { .. } => scanpci(),
            Command::Top { .. } => top(),
            Command::Uptime { .. } => uptime(),
            Command::Sleep { seconds, .. } => cli_sleep(seconds),
            Command::Shutdown { .. } => shutdown(),
            Command::Ping { ip_address, count, timeout, .. } => ping(ip_address.0, count, timeout),
            Command::Ip { subcommand, .. } => {
                set_max_verbosity(subcommand.get_verbosity());

                match subcommand {
                    IpCommand::Interface(subcommand) | IpCommand::I(subcommand) => match subcommand {
                        None => ip_interface_show(),
                        Some(subcommand) => match subcommand {
                            IpInterfaceCommand::Show { .. } => ip_interface_show(),
                        }
                    },
                    IpCommand::Address(subcommand) | IpCommand::A(subcommand) => match subcommand {
                        None => Ok(()),
                        Some(subcommand) => match subcommand {
                            IpAddressCommand::Add { address, interface_name, .. } => ip_address_add(address.0, interface_name.0),
                            IpAddressCommand::Delete { address, interface_name, .. } => ip_address_delete(address.0, interface_name.0),
                        }
                    },
                    IpCommand::Route(subcommand) | IpCommand::R(subcommand) => match subcommand {
                        None => ip_route_show(),
                        Some(subcommand) => match subcommand {
                            IpRouteCommand::Show { .. } => ip_route_show(),
                            IpRouteCommand::Add { address, interface_name, gateway, .. } => ip_route_add(address.0, interface_name.0, gateway.0),
                            IpRouteCommand::Delete { address, interface_name, .. } => ip_route_delete(address.0, interface_name.0)
                        }
                    }
                }
            }
        }
    });

    let result = cli.process_byte::<Command<'_>, _>(byte, &mut command_processor);

    if let Err(error) = result {
        cli
            .write(|w| {
                w.write_str(format!("{} {}", "Error:".red(), error).as_ref())
            })
            .unwrap();
    }
}

fn set_max_verbosity(verbosity: &Option<Verbosity>) {
    match verbosity {
        None => set_max_level(LevelFilter::Error),
        Some(verbosity) => set_max_level(verbosity.level)
    }
}