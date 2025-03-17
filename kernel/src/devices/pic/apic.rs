use crate::interrupts::interrupt::InterruptIndex;
use crate::memory::allocator::BOOT_INFO_FRAME_ALLOCATOR;
use crate::memory::tables::MAPPER;
use acpi::platform::interrupt;
use alloc::alloc::Global;
use core::ops::DerefMut;
use core::ptr;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use crate::devices::pic::pic::{PicType, PIC};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(isize)]
#[allow(dead_code)]
pub enum APICOffset {
    R0x00 = 0x0,      // RESERVED = 0x00
    R0x10 = 0x10,     // RESERVED = 0x10
    Ir = 0x20,        // ID Register
    Vr = 0x30,        // Version Register
    R0x40 = 0x40,     // RESERVED = 0x40
    R0x50 = 0x50,     // RESERVED = 0x50
    R0x60 = 0x60,     // RESERVED = 0x60
    R0x70 = 0x70,     // RESERVED = 0x70
    Tpr = 0x80,       // Text Priority Register
    Apr = 0x90,       // Arbitration Priority Register
    Ppr = 0xA0,       // Processor Priority Register
    Eoi = 0xB0,       // End of Interrupt
    Rrd = 0xC0,       // Remote Read Register
    Ldr = 0xD0,       // Logical Destination Register
    Dfr = 0xE0,       // DFR
    Svr = 0xF0,       // Spurious (Interrupt) Vector Register
    Isr1 = 0x100,     // In-Service Register 1
    Isr2 = 0x110,     // In-Service Register 2
    Isr3 = 0x120,     // In-Service Register 3
    Isr4 = 0x130,     // In-Service Register 4
    Isr5 = 0x140,     // In-Service Register 5
    Isr6 = 0x150,     // In-Service Register 6
    Isr7 = 0x160,     // In-Service Register 7
    Isr8 = 0x170,     // In-Service Register 8
    Tmr1 = 0x180,     // Trigger Mode Register 1
    Tmr2 = 0x190,     // Trigger Mode Register 2
    Tmr3 = 0x1A0,     // Trigger Mode Register 3
    Tmr4 = 0x1B0,     // Trigger Mode Register 4
    Tmr5 = 0x1C0,     // Trigger Mode Register 5
    Tmr6 = 0x1D0,     // Trigger Mode Register 6
    Tmr7 = 0x1E0,     // Trigger Mode Register 7
    Tmr8 = 0x1F0,     // Trigger Mode Register 8
    Irr1 = 0x200,     // Interrupt Request Register 1
    Irr2 = 0x210,     // Interrupt Request Register 2
    Irr3 = 0x220,     // Interrupt Request Register 3
    Irr4 = 0x230,     // Interrupt Request Register 4
    Irr5 = 0x240,     // Interrupt Request Register 5
    Irr6 = 0x250,     // Interrupt Request Register 6
    Irr7 = 0x260,     // Interrupt Request Register 7
    Irr8 = 0x270,     // Interrupt Request Register 8
    Esr = 0x280,      // Error Status Register
    R0x290 = 0x290,   // RESERVED = 0x290
    R0x2A0 = 0x2A0,   // RESERVED = 0x2A0
    R0x2B0 = 0x2B0,   // RESERVED = 0x2B0
    R0x2C0 = 0x2C0,   // RESERVED = 0x2C0
    R0x2D0 = 0x2D0,   // RESERVED = 0x2D0
    R0x2E0 = 0x2E0,   // RESERVED = 0x2E0
    LvtCmci = 0x2F0,  // LVT Corrected Machine Check Interrupt (CMCI) Register
    Icr1 = 0x300,     // Interrupt Command Register 1
    Icr2 = 0x310,     // Interrupt Command Register 2
    LvtT = 0x320,     // LVT Timer Register
    LvtTsr = 0x330,   // LVT Thermal Sensor Register
    LvtPmcr = 0x340,  // LVT Performance Monitoring Counters Register
    LvtLint0 = 0x350, // LVT LINT0 Register
    LvtLint1 = 0x360, // LVT LINT1 Register
    LvtE = 0x370,     // LVT Error Register
    Ticr = 0x380,     // Initial Count Register (for Timer)
    Tccr = 0x390,     // Current Count Register (for Timer)
    R0x3A0 = 0x3A0,   // RESERVED = 0x3A0
    R0x3B0 = 0x3B0,   // RESERVED = 0x3B0
    R0x3C0 = 0x3C0,   // RESERVED = 0x3C0
    R0x3D0 = 0x3D0,   // RESERVED = 0x3D0
    Tdcr = 0x3E0,     // Divide Configuration Register (for Timer)
    R0x3F0 = 0x3F0,   // RESERVED = 0x3F0
}

pub struct Apic(*mut u32);

unsafe impl Send for Apic {}
unsafe impl Sync for Apic {}

impl Apic {
    pub fn init_apic(apic: interrupt::Apic<Global>) {
        Apic::init_io_apic(apic.io_apics[0].address as u64);
        Apic::init_local_apic(apic.local_apic_address);
        Apic::disable_pic();
    }
    
    fn init_local_apic(local_apic_addr: u64) {
        PIC.call_once(|| Mutex::new(PicType::APIC(Apic(ptr::null_mut()))));
        let mut pic = PIC.get().unwrap().lock();
        let local_apic = pic.unwrap_apic();

        let virt_addr = Apic::map_apic(local_apic_addr);
        local_apic.0 = virt_addr.as_mut_ptr::<u32>();

        unsafe {
            local_apic.init_timer();
            local_apic.init_keyboard();
        }
    }

    fn init_io_apic(io_apic_address: u64) {
        let virt_addr = Apic::map_apic(io_apic_address);

        let io_apic_pointer = virt_addr.as_mut_ptr::<u32>();

        unsafe {
            io_apic_pointer
                .offset(0)
                .write_volatile(0x12);
            io_apic_pointer
                .offset(4)
                .write_volatile(InterruptIndex::Keyboard.as_u8() as u32);
        }
    }

    unsafe fn init_timer(&self) {
        let svr = self.0.offset(APICOffset::Svr as isize / 4);
        svr.write_volatile(svr.read_volatile() | 0x100); // Set bit 8

        let lvt_lint1 = self.0.offset(APICOffset::LvtT as isize / 4);
        lvt_lint1.write_volatile(0x20 | (1 << 17)); // Vector 0x20, periodic mode

        let tdcr = self.0.offset(APICOffset::Tdcr as isize / 4);
        tdcr.write_volatile(0x3); // Divide by 16 mode

        let ticr = self.0.offset(APICOffset::Ticr as isize / 4);
        ticr.write_volatile(0x100000); // An arbitrary value for the initial value of the timer
    }

    unsafe fn init_keyboard(&self) {
        let keyboard_register = self.0.offset(APICOffset::LvtLint1 as isize / 4);
        keyboard_register.write_volatile(InterruptIndex::Keyboard as u8 as u32);
    }

    fn map_apic(physical_address: u64) -> VirtAddr {
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(physical_address));
        let frame = PhysFrame::containing_address(PhysAddr::new(physical_address));
        
        let mut mapper = MAPPER.get().unwrap().lock();
        let mut frame_allocator = BOOT_INFO_FRAME_ALLOCATOR.lock();

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator.deref_mut())
                .expect("APIC mapping failed")
                .flush();
        }

        page.start_address()
    }

    /// Disable any unneeded PIC features, such as timer or keyboard to prevent it from firing interrupts
    fn disable_pic() {
        unsafe {
            // PIC2 (Slave PIC)
            Port::<u8>::new(0x21).write(0xFF);
            Port::<u8>::new(0xA1).write(0xFF);
        }
    }

    pub unsafe fn end_interrupt(&mut self) {
        self.0
            .offset(APICOffset::Eoi as isize / 4)
            .write_volatile(0)
    }
}