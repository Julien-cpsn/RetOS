use crate::println;
use core::fmt::Arguments;
use goolog::log::Level;
use yansi::{Color, Paint};

pub fn print_log(timestamp: &str, target: &str, level: Level, args: &Arguments) {
    let color = match level {
        Level::Error => Color::Red,
        Level::Warn => Color::Yellow,
        Level::Info => Color::Green,
        Level::Debug => Color::Blue,
        Level::Trace => Color::Black
    };
    println!("[{} | {} | {}] {}", timestamp, level.white().bg(color), target, args);
}