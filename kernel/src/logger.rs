use crate::clock::Clock;
use crate::println;
use core::fmt::Arguments;
use goolog::log::Level;
use yansi::{Color, Paint};

pub fn print_log(target: &str, level: Level, args: &Arguments) {
    let color = match level {
        Level::Error => Color::Red,
        Level::Warn => Color::Yellow,
        Level::Info => Color::Green,
        Level::Debug => Color::Blue,
        Level::Trace => Color::Black
    };
    let timestamp = Clock::format();
    println!("[{} | {} | {}] {}", timestamp, level.white().bg(color), target, args);
}