use crate::devices::pic::pic::{PicType, PIC, PIC_1_OFFSET, PIC_2_OFFSET};
use pic8259::ChainedPics;
use spin::Mutex;

pub fn init_legacy_pics() {
    PIC.call_once(|| Mutex::new(PicType::PICS(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) })));
    let mut pic = PIC.get().unwrap().lock();
    unsafe { pic.unwrap_pics().initialize() };
}