use crate::clock::tick_handler;
use crate::devices::pic::pic::PIC;
use crate::devices::network_controller::NETWORK_CONTROLLER;
use crate::interrupts::gdt;
use crate::interrupts::interrupt::InterruptIndex;
use crate::{hlt_loop, println, task};
use pc_keyboard::layouts::{AnyLayout, Us104Key};
use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1};
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

    Mutex::new(idt)
});

pub static KEYBOARD: Lazy<RwLock<Keyboard<AnyLayout, ScancodeSet1>>> = Lazy::new(|| RwLock::new(Keyboard::new(ScancodeSet1::new(), AnyLayout::Us104Key(Us104Key), HandleControl::Ignore)));

pub fn init_idt() {
    let idt = IDT.lock();
    unsafe { idt.load_unsafe() };
}

pub fn register_interrupt(offset: u32, interrupt: InterruptIndex, handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
    let mut idt = IDT.lock();
    idt[interrupt.as_u8()].set_handler_fn(handler);
    unsafe { idt.load_unsafe() };

    PIC
        .get()
        .unwrap()
        .lock()
        .register_interrupt(offset, interrupt)
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    tick_handler();

    PIC
        .get()
        .unwrap()
        .lock()
        .end_interrupt(InterruptIndex::Timer.as_u8());
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    task::keyboard::add_scancode(scancode);

    PIC
        .get()
        .unwrap()
        .lock()
        .end_interrupt(InterruptIndex::Keyboard.as_u8());
}

pub extern "x86-interrupt" fn network_packet_handler(_stack_frame: InterruptStackFrame) {
    PIC
        .get()
        .unwrap()
        .lock()
        .end_interrupt(InterruptIndex::NetworkPacket.as_u8());
    
    NETWORK_CONTROLLER
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
