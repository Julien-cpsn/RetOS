use crate::add_verbosity;
use crate::terminal::commands::clear::clear;
use crate::terminal::commands::echo::echo;
use crate::terminal::commands::keyboard::{change_layout, KeyboardLayoutArg};
use crate::terminal::commands::ps::ps;
use crate::terminal::commands::shutdown::shutdown;
use crate::terminal::commands::uptime::uptime;
use crate::terminal::commands::lspci::lspci;
use crate::terminal::commands::scanpci::scanpci;
use crate::terminal::error::CliError;
use crate::terminal::terminal::TerminalBuffer;
use embedded_cli::cli::Cli;
use embedded_cli::Command;
use goolog::log::{set_max_level, LevelFilter};
use crate::terminal::commands::top::top;

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

        /// Change the keyboard layout
        Keyboard {
            /// Keyboard layout to use
            layout: KeyboardLayoutArg,
        },
        
        /// List PCI devices
        Lspci,
        
        /// Enforces PCI device scan
        Scanpci,

        /// Print details about system resources usage
        Top,
        
        /// Print for how much time the system is running
        Uptime,

        /// Shutdown the operating system
        Shutdown,
    }
}

pub fn handle_command(cli: &mut Cli<&mut TerminalBuffer, CliError, [u8; 100], [u8; 100]>, byte: u8) {
    cli.process_byte::<Command<'_>, _>(
        byte,
        &mut Command::processor(|_cli, command| {
            match command.get_verbosity() {
                None => set_max_level(LevelFilter::Error),
                Some(verbosity) => set_max_level(verbosity.level)
            }

            match command {
                Command::Echo { text, .. } => echo(text),
                Command::Clear { .. } => clear(),
                Command::Ps { .. } => ps(),
                Command::Keyboard { layout, .. } => change_layout(layout),
                Command::Lspci { .. } => lspci(),
                Command::Scanpci { .. } => scanpci(),
                Command::Top { .. } => top(),
                Command::Uptime { .. } => uptime(),
                Command::Shutdown { .. } => shutdown(),
            }
        }),
    )
        .expect("CLI processing failed");
}