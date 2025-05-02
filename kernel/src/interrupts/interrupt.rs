use crate::devices::pic::pic::PIC_1_OFFSET;
use crate::devices::serial::SERIAL1_IRQ;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = 2,
    Serial1 = SERIAL1_IRQ,
    NetworkPacket,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}