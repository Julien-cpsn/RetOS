#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(type_alias_impl_trait)]
#![feature(allocator_api)]

extern crate alloc;

use crate::interrupts::{gdt, idt};

pub mod printer;
pub mod interrupts;
pub mod memory;
pub mod allocator;
pub mod task;
pub mod terminal;
pub mod clock;
pub mod logger;
pub mod devices;

pub fn init() {
    print!("\t> Initializing GDT... ");
    gdt::init_gdt();
    println!("initialized!");

    print!("\t> Initializing IDT... ");
    idt::init_idt();
    println!("initialized!");

    print!("\t> Initializing PICS... ");
    unsafe { interrupts::pics::PICS.write().initialize() };
    println!("initialized!");

    print!("\t> Allocating heap... ");
    allocator::init_allocator();
    println!("allocated!");

    print!("\t> Enabling interrupts... ");
    x86_64::instructions::interrupts::enable();
    println!("enabled!");
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}