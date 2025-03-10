use crate::printer::buffer::WRITER;
use crate::printer::color::{Color, DEFAULT_BACKGROUND, DEFAULT_FOREGROUND};
use crate::{print, println, set_background, set_foreground};
use core::fmt::Arguments;
use goolog::log::Level;

pub fn print_log(timestamp: &str, target: &str, level: Level, args: &Arguments) {
    print!("[{timestamp} | ");
    let color = match level {
        Level::Error => Color::RED,
        Level::Warn => Color::YELLOW,
        Level::Info => Color::GREEN,
        Level::Debug => Color::BLUE,
        Level::Trace => DEFAULT_BACKGROUND,
    };
    let white = Color::WHITE;
    set_foreground!(white);
    set_background!(color);
    print!("{level}");
    set_foreground!(DEFAULT_FOREGROUND);
    set_background!(DEFAULT_BACKGROUND);
    println!(" | {target}] {args}");
}

