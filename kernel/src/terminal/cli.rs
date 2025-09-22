use crate::{print, println};
use crate::printer::buffer::{Writer, BORDER_PADDING};
use crate::terminal::args::{CliArgs, Commands};
use crate::terminal::commands::clear::clear;
use crate::terminal::commands::echo::{echo, EchoCommand};
use crate::terminal::commands::ip::address::{ip_address_add, ip_address_delete, IpAddressAddCommand, IpAddressCommand, IpAddressDeleteCommand};
use crate::terminal::commands::ip::interface::{ip_interface_show, IpInterfaceCommand};
use crate::terminal::commands::ip::ip::IpCommand;
use crate::terminal::commands::ip::route::{ip_route_add, ip_route_delete, ip_route_show, IpRouteAddCommand, IpRouteCommand, IpRouteDeleteCommand};
use crate::terminal::commands::keyboard::change_layout;
use crate::terminal::commands::lspci::lspci;
use crate::terminal::commands::ping::{ping, PingCommand};
use crate::terminal::commands::ps::ps;
use crate::terminal::commands::scanpci::scanpci;
use crate::terminal::commands::shutdown::shutdown;
use crate::terminal::commands::sleep::cli_sleep;
use crate::terminal::commands::top::top;
use crate::terminal::commands::uptime::uptime;
use crate::terminal::custom_arguments::verbosity::verbosity_to_level_filter;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Write;
use goolog::log::set_max_level;
use no_std_clap_core::parser::Parser;
use spin::RwLock;

pub struct Cli {
    prompt: String,
    prompt_length: usize,
    line: String,
    cursor_index: usize,
    writer: Arc<RwLock<Writer>>,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl Cli {
    pub fn new(prompt: String, prompt_length: usize, writer: Arc<RwLock<Writer>>) -> Self {
        writer.write().write_str(&prompt).unwrap();

        Self {
            prompt,
            prompt_length,
            line: String::new(),
            writer,
            cursor_index: 0,
            history: Vec::new(),
            history_index: None,
        }
    }

    pub fn reset_line(&mut self) {
        self.print_current_line(false, true);
    }

    fn print_current_line(&mut self, line_from_history: bool, show_cursor: bool) {
        let mut writer = self.writer.write();

        writer.clear_line();
        writer.write_str(&self.prompt).unwrap();

        let line_length = self.line.len();

        if self.cursor_index > line_length || (line_from_history && self.cursor_index == 0) {
            self.cursor_index = line_length;
        }

        // Write the full line
        writer.write_str(&self.line).unwrap();

        // Now compute cursor_x, cursor_y
        writer.cursor_x = BORDER_PADDING + (self.prompt_length * Writer::column_width()) + (self.cursor_index * Writer::column_width());
        writer.cursor_y = writer.y;

        writer.show_cursor = show_cursor;
        writer.draw_cursor();
    }

    pub fn handle_scancode(&mut self, scancode: u8) -> Option<String> {
        match scancode {
            b'\n' => match self.line.is_empty() {
                true => {
                    self.line.clear();
                    self.print_current_line(false, false);
                    self.writer.write().newline();
                    self.history_index = None;
                    self.cursor_index = 0;
                }
                false => {
                    self.print_current_line(false, false);
                    self.writer.write().newline();

                    self.history_index = None;
                    self.cursor_index = 0;

                    let command = self.line.clone();
                    self.line.clear();

                    if self.history.last().map(|last_command| last_command != &command).unwrap_or(true) {
                        self.history.push(command.clone());
                    }

                    return Some(command);
                }
            },
            // Tab
            0x9 => {

            },
            // Backspace
            0x8 => {
                if self.cursor_index > 0 {
                    self.cursor_index -= 1;
                    self.line.remove(self.cursor_index);
                }
            },
            // Delete
            0x7F => {
                if self.cursor_index < self.line.len() {
                    self.line.remove(self.cursor_index);
                }
            },
            _ => {
                //println!("{:X}", scancode);
                //println!("{}", self.cursor_index);
                self.line.insert(self.cursor_index, scancode as char);
                self.cursor_index += 1;
            }
        }

        self.print_current_line(false, true);

        None
    }

    pub fn previous_command(&mut self) {
        match &mut self.history_index {
            None => if !self.history.is_empty() {
                self.history_index = Some(self.history.len() - 1);
            }
            else {
                return;
            }
            Some(index) => if *index > 0 {
                *index = index.saturating_sub(1);
            }
            else {
                return;
            }
        }

        self.line = self.history[self.history_index.unwrap()].clone();

        self.print_current_line(true, true)
    }

    pub fn next_command(&mut self) {
        if let Some(index) = &mut self.history_index {
            if *index + 1 == self.history.len() {
                self.line.clear();
                self.history_index = None;
            }
            else {
                *index += 1;
                self.line = self.history[*index].clone();
            }
        }
        else {
            return;
        }

        self.print_current_line(true, true)
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
            self.print_current_line(false, true);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_index < self.line.len() {
            self.cursor_index += 1;
            self.print_current_line(false, true);
        }
    }
}

pub fn handle_command(command: Commands) {
    let result = match command {
        Commands::Help => {
            let help = CliArgs::get_help();
            print!("{}", help);
            Ok(())
        },
        Commands::Echo(EchoCommand { text }) => echo(&text),
        Commands::Clear => clear(),
        Commands::Ps => ps(),
        Commands::Keyboard { layout } => change_layout(layout),
        Commands::Lspci => lspci(),
        Commands::Scanpci => scanpci(),
        Commands::Top => top(),
        Commands::Uptime => uptime(),
        Commands::Sleep { seconds, .. } => cli_sleep(seconds),
        Commands::Shutdown => shutdown(),
        Commands::Ping (PingCommand { ip_address, count, timeout }) => ping(ip_address.0, count, timeout),
        Commands::Ip(subcommand) => {
            match subcommand {
                IpCommand::Interface(subcommand) | IpCommand::I(subcommand) => match subcommand {
                    None => ip_interface_show(),
                    Some(subcommand) => match subcommand {
                        IpInterfaceCommand::Show => ip_interface_show(),
                    }
                },
                IpCommand::Address(subcommand) | IpCommand::A(subcommand) => match subcommand {
                    None => Ok(()),
                    Some(subcommand) => match subcommand {
                        IpAddressCommand::Add(IpAddressAddCommand { address, interface_name }) => ip_address_add(address.0, &interface_name.0),
                        IpAddressCommand::Delete(IpAddressDeleteCommand { address, interface_name }) => ip_address_delete(address.0, &interface_name.0),
                    }
                },
                IpCommand::Route(subcommand) | IpCommand::R(subcommand) => match subcommand {
                    None => ip_route_show(),
                    Some(subcommand) => match subcommand {
                        IpRouteCommand::Show => ip_route_show(),
                        IpRouteCommand::Add (IpRouteAddCommand { address, interface_name, gateway }) => ip_route_add(address.0, &interface_name.0, gateway.0),
                        IpRouteCommand::Delete(IpRouteDeleteCommand { address, interface_name }) => ip_route_delete(address.0, &interface_name.0)
                    }
                }
            }
        }
    };

    if let Err(error) = result {
        use yansi::Paint;
        println!("{} {}", "Error:".red(), error)
    }
}

pub fn set_max_verbosity(verbosity: usize) {
    let level_filter = verbosity_to_level_filter(verbosity);
    set_max_level(level_filter)
}