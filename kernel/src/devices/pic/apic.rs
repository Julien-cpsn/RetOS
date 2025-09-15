use crate::devices::pic::pic::{PicType, PIC};
use crate::interrupts::interrupt::InterruptIndex;
use crate::memory::allocator::BOOT_INFO_FRAME_ALLOCATOR;
use crate::memory::tables::MAPPER;
use acpi::platform::interrupt;
use alloc::alloc::Global;
use core::ops::DerefMut;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

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

#[derive(Default)]
pub struct Apic {
    io: *mut u32,
    local: *mut u32
}

unsafe impl Send for Apic {}
unsafe impl Sync for Apic {}

impl Apic {
    pub fn init_apic(apic: interrupt::Apic<Global>) {
        PIC.call_once(|| Mutex::new(PicType::APIC(Apic::default())));
        Apic::init_local_apic(apic.local_apic_address);
        Apic::init_io_apic(apic.io_apics[0].address as u64);
        Apic::disable_legacy_pic();
    }
    
    fn init_local_apic(local_apic_addr: u64) {
        let mut pic = PIC.get().unwrap().lock();
        let apic = pic.unwrap_apic();

        let virt_addr = Apic::map_apic(local_apic_addr);
        apic.local = virt_addr.as_mut_ptr::<u32>();

        unsafe {
            apic.init_timer();
        }
    }

    fn init_io_apic(io_apic_address: u64) {
        let mut pic = PIC.get().unwrap().lock();
        let apic = pic.unwrap_apic();
        
        let virt_addr = Apic::map_apic(io_apic_address);
        apic.io = virt_addr.as_mut_ptr::<u32>();

        let apic_id = apic.local_apic_id();

        apic.register_ioapic_interrupt(1, InterruptIndex::Keyboard.as_u8(), apic_id);
        apic.register_ioapic_interrupt(4, InterruptIndex::Serial1.as_u8(), apic_id);
    }

    pub fn local_apic_id(&self) -> u8 {
        unsafe {
            // APIC ID Register is at offset 0x20 (divided by 4 = index 0x8)
            let ir = self.local.offset(APICOffset::Ir as isize / 4);
            let value = ir.read_volatile();
            ((value >> 24) & 0xFF) as u8
        }
    }

    pub fn register_ioapic_interrupt(&self, irq: u8, vector: u8, dest_apic_id: u8) {
        // IOAPIC register indices:
        // redir_index_low  = 0x10 + 2*irq
        // redir_index_high = 0x10 + 2*irq + 1
        let redir_low_index = 0x10 + (irq as u32 * 2);
        let redir_high_index = redir_low_index + 1;

        let low = (vector as u32)
            | (0 << 8)     // delivery = Fixed
            | (0 << 11)    // dest mode = Physical
            | (0 << 13)    // polarity = active-low
            | (0 << 15)    // trigger = level-triggered
            | (0 << 16);   // mask = 0 (enabled)

        // High dword: destination APIC ID occupies bits 24..31
        let high = (dest_apic_id as u32) << 24;

        unsafe {
            // select low register
            self.io.offset(0).write_volatile(redir_low_index);
            // write low dword to IOWIN (IOREGSEL + 0x10 == IOWIN; your offset(4) is correct)
            self.io.offset(4).write_volatile(low);

            // select high register
            self.io.offset(0).write_volatile(redir_high_index);
            // write high dword
            self.io.offset(4).write_volatile(high);
        }
    }

    unsafe fn init_timer(&self) {
        let svr = self.local.offset(APICOffset::Svr as isize / 4);
        svr.write_volatile(svr.read_volatile() | 0x100); // enable LAPIC

        let lvt_timer = self.local.offset(APICOffset::LvtT as isize / 4);
        lvt_timer.write_volatile(InterruptIndex::Timer as u32 | (1 << 17)); // Vector 0x20, periodic mode

        let tdcr = self.local.offset(APICOffset::Tdcr as isize / 4);
        tdcr.write_volatile(0x3); // Divide by 16

        let ticr = self.local.offset(APICOffset::Ticr as isize / 4);
        ticr.write_volatile(0x100000); // initial count
    }

    fn map_apic(physical_address: u64) -> VirtAddr {
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(physical_address));
        let frame = PhysFrame::containing_address(PhysAddr::new(physical_address));
        
        let mut mapper = MAPPER.get().unwrap().write();
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
    fn disable_legacy_pic() {
        unsafe {
            // PIC2 (Slave PIC)
            Port::<u8>::new(0x21).write(0xFF);
            Port::<u8>::new(0xA1).write(0xFF);
        }
    }

    pub fn end_interrupt(&mut self) {
        unsafe {
            self
                .local
                .offset(APICOffset::Eoi as isize / 4)
                .write_volatile(0)
        }
    }

    pub fn get_interrupt_vector(&self) -> Option<u8> {
        // There are 8 ISR registers, each 32 bits wide, covering vectors 0-255
        // ISR1 covers vectors 0-31, ISR2 covers 32-63, etc.
        let isr_registers = [
            APICOffset::Isr1,
            APICOffset::Isr2,
            APICOffset::Isr3,
            APICOffset::Isr4,
            APICOffset::Isr5,
            APICOffset::Isr6,
            APICOffset::Isr7,
            APICOffset::Isr8,
        ];

        unsafe {
            // Check each ISR register for bits that are set
            for (index, &register) in isr_registers.iter().enumerate() {
                let register_value = self.local.offset(register as isize / 4).read_volatile();

                // If the register has any bits set, find the highest priority one
                if register_value != 0 {
                    // Calculate which bit is set (using trailing zeros)
                    let bit_position = register_value.trailing_zeros();

                    // Calculate the vector number
                    // Each register covers 32 vectors, so we multiply the register index by 32
                    // and add the bit position
                    let vector = (index as u8 * 32) + bit_position as u8;

                    return Some(vector);
                }
            }
        }

        // No interrupt is in service
        None
    }
}