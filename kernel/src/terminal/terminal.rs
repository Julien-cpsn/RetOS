use crate::printer::buffer::WRITER;
use alloc::string::String;
use core::fmt::Write;

pub struct TerminalBuffer;

impl embedded_cli::__private::io::Write for TerminalBuffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        write_to_writer(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn write_to_writer(buf: &[u8]) {
    let mut writer = WRITER.write();
    // Debugger
    //writer.write_str(&alloc::format!("{:x?}\n", buf)).unwrap();

    // See https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
    match buf {
        // Unknown?
        [0x1B, 0x5B, 0x50] => {}
        // Backspace
        [0x1B, 0x5B, 0x44] => writer.erase_char(),
        // Erase line sequence
        // ESC[2K
        [0x1B, 0x5B, 0x32, 0x4B] => writer.clear_line(),
        _ => writer.write_str(&String::from_utf8_lossy(buf)).unwrap(),
    }
}

