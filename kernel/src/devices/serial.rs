use spin::{Lazy, RwLock};
use uart_16550::SerialPort;

pub const SERIAL1_BASE: u16 = 0x3F8;
pub const SERIAL1_IRQ: u8 = 0x4;

pub static SERIAL1: Lazy<RwLock<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(SERIAL1_BASE) };
    serial_port.init();
    RwLock::new(serial_port)
});