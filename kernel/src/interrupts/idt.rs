use crate::clock::tick_handler;
use crate::devices::network::interrupt::{process_pending_network_irqs, PENDING_NETWORK_IRQS};
use crate::devices::network::manager::NETWORK_DEVICES_INTERRUPT_IRQ;
use crate::devices::pic::pic::PIC;
use crate::devices::serial::SERIAL1;
use crate::interrupts::gdt;
use crate::interrupts::interrupt::InterruptIndex;
use crate::{hlt_loop, println, task};
use pc_keyboard::layouts::{AnyLayout, Us104Key};
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};
use spin::{Lazy, Mutex, RwLock};
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

/// Interrupt Descriptor Table.
/// Data structure used by the x86 architecture to implement an interrupt vector table.
pub static IDT: Lazy<Mutex<InterruptDescriptorTable>> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);

    unsafe {
        idt
            .double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::Serial1.as_u8()].set_handler_fn(serial_interrupt_handler);

    idt[NETWORK_DEVICES_INTERRUPT_IRQ].set_handler_fn(network_packet_handler_0);
    idt[NETWORK_DEVICES_INTERRUPT_IRQ + 1].set_handler_fn(network_packet_handler_1);
    idt[NETWORK_DEVICES_INTERRUPT_IRQ + 2].set_handler_fn(network_packet_handler_2);
    idt[NETWORK_DEVICES_INTERRUPT_IRQ + 3].set_handler_fn(network_packet_handler_3);

    Mutex::new(idt)
});

pub static KEYBOARD: Lazy<RwLock<Keyboard<AnyLayout, ScancodeSet1>>> = Lazy::new(|| RwLock::new(Keyboard::new(ScancodeSet1::new(), AnyLayout::Us104Key(Us104Key), HandleControl::Ignore)));

pub fn init_idt() {
    let idt = IDT.lock();
    unsafe { idt.load_unsafe() };
}

/*
pub fn register_interrupt(offset: u8, interrupt: u8, handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
    let mut idt = IDT.lock();
    idt[interrupt].set_handler_fn(handler);
    unsafe { idt.load_unsafe() };

    PIC
        .get()
        .unwrap()
        .lock()
        .register_interrupt(offset, interrupt)
}*/

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    tick_handler();

    // TODO: can be placed in a task when SMP is on
    process_pending_network_irqs();

    PIC
        .get()
        .unwrap()
        .lock()
        .end_interrupt();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    {
        let mut keyboard = KEYBOARD.write();
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                task::keyboard::add_key(key);
            }
        }
    }

    PIC
        .get()
        .unwrap()
        .lock()
        .end_interrupt();
}


extern "x86-interrupt" fn serial_interrupt_handler(_stack_frame: InterruptStackFrame) {
    {
        let scancode = SERIAL1.write().receive();
        let key = match scancode {
            0x20 => DecodedKey::Unicode(' '),
            0x09 => DecodedKey::Unicode('\t'),
            0x0D => DecodedKey::Unicode('\n'),
            0x0A => DecodedKey::Unicode('\n'),
            0x08 => DecodedKey::RawKey(KeyCode::Backspace),
            0x21..=0x7E => DecodedKey::Unicode(scancode as char),
            _ => return
        };

        task::keyboard::add_key(key);
    }

    PIC
        .get()
        .unwrap()
        .lock()
        .end_interrupt();
}

pub extern "x86-interrupt" fn network_packet_handler_0(_stack_frame: InterruptStackFrame) {
    network_packet_handler(0xB);
}

pub extern "x86-interrupt" fn network_packet_handler_1(_stack_frame: InterruptStackFrame) {
    network_packet_handler(0xB + 1);
}

pub extern "x86-interrupt" fn network_packet_handler_2(_stack_frame: InterruptStackFrame) {
    network_packet_handler(0xB + 2);
}

pub extern "x86-interrupt" fn network_packet_handler_3(_stack_frame: InterruptStackFrame) {
    network_packet_handler(0xB + 3);
}


fn network_packet_handler(interrupt_line: u8) {
    //println!("Received packet");

    PENDING_NETWORK_IRQS.push(interrupt_line);

    PIC
        .get()
        .unwrap()
        .lock()
        .end_interrupt();
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:?}", stack_frame);
    hlt_loop();
}
