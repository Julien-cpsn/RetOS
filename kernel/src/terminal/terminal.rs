use crate::printer::buffer::WRITER;

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

#[inline]
pub fn write_to_writer(buf: &[u8]) {
    let mut writer = WRITER.write();
    // Debugger
    //writer.write_str(&alloc::format!("{:x?}\n", buf)).unwrap();

    for byte in buf {
        writer.write_byte(*byte);
    }
}