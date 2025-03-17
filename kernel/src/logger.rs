use crate::println;
use colorz::ansi::AnsiColor;
use colorz::Colorize;
use core::fmt::Arguments;
use goolog::log::Level;

pub fn print_log(timestamp: &str, target: &str, level: Level, args: &Arguments) {
    let color = match level {
        Level::Error => AnsiColor::Red,
        Level::Warn => AnsiColor::Yellow,
        Level::Info => AnsiColor::Green,
        Level::Debug => AnsiColor::Blue,
        Level::Trace => AnsiColor::Black
    };
    println!("[{} | {} | {}] {}", timestamp, level.white().bg(color), target, args);
}