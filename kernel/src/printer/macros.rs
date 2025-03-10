use crate::printer::buffer::WRITER;
use core::fmt;

#[macro_export]
macro_rules! println {
    () => ($crate::printer::macros::_print(format_args!("\n")));
    ($($arg:tt)*) => ($crate::printer::macros::_print(format_args!("{}\n", format_args!($($arg)*))));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::printer::macros::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.write().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! set_foreground {
    ($arg:tt) => {
        WRITER.write().fg_color = $arg;
    };
}

#[macro_export]
macro_rules! set_background {
    ($arg:tt) => {
        WRITER.write().bg_color = $arg;
    };
}

