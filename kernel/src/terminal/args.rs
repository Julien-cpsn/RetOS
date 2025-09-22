use crate::terminal::commands::echo::EchoCommand;
use crate::terminal::commands::ip::ip::IpCommand;
use crate::terminal::commands::keyboard::KeyboardLayout;
use crate::terminal::commands::ping::PingCommand;
use no_std_clap_core::arg::arg_info::ArgInfo;
use no_std_clap_macros::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "RetOS", author = "Julien-cpsn", version = "0.1.0", about = "A Router Network Operating System.")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, count, global)]
    pub verbose: usize,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Display this help message
    Help,

    /// Echoes the following argument
    Echo(EchoCommand),

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
        layout: KeyboardLayout,
    },

    /// List PCI devices
    Lspci,

    /// Enforces a PCI device scan
    Scanpci,

    /// Ping an IP address
    Ping(PingCommand),

    /// Network commands
    #[command(subcommand)]
    Ip(IpCommand)
}