use acpi::{AcpiTables, InterruptModel};
use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x86_64::VirtAddr;
use crate::devices::acpi::AcpiHandlerImpl;
use crate::devices::pic::apic::Apic;
use crate::devices::pic::legacy::init_legacy_pics;
use crate::interrupts::interrupt::InterruptIndex;
use crate::print;

pub static PIC: Once<Mutex<PicType>> = Once::new();

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub enum PicType {
    APIC(Apic),
    PICS(ChainedPics)
}

impl PicType {
    pub fn unwrap_apic(&mut self) -> &mut Apic {
        match self {
            PicType::APIC(apic) => apic,
            PicType::PICS(_) => panic!("Unwrapping PICS")
        }
    }

    pub fn unwrap_pics(&mut self) -> &mut ChainedPics {
        match self {
            PicType::APIC(_) => panic!("Unwrapping APIC"),
            PicType::PICS(pics) => pics
        }
    }

    pub fn register_interrupt(&self, offset: u32, index: InterruptIndex) {
        match self {
            PicType::APIC(apic) => apic.register_interrupt(offset, index),
            PicType::PICS(_) => todo!()
        }
    }
    
    pub fn end_interrupt(&mut self, interrupt_id: u8) {
        unsafe {
            match self {
                PicType::APIC(apic) => apic.end_interrupt(),
                PicType::PICS(pics) => pics.notify_end_of_interrupt(interrupt_id)
            }
        }
    }
}

pub fn init_pic(rsdp: usize, physical_memory_offset: VirtAddr) {
    let handler = AcpiHandlerImpl::new(physical_memory_offset);
    let acpi_tables = unsafe { AcpiTables::from_rsdp(handler, rsdp).expect("Failed to parse ACPI tables") };

    let platform_info = acpi_tables
        .platform_info()
        .expect("Failed to get platform info");

    match platform_info.interrupt_model {
        InterruptModel::Apic(apic) => {
            print!("APIC... ");
            Apic::init_apic(apic);
        },
        InterruptModel::Unknown => {
            print!("legacy PICs... ");
            init_legacy_pics();
        },
        _ => {}
    }
}