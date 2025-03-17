#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(type_alias_impl_trait)]
#![feature(allocator_api)]

extern crate alloc;

use interrupts::{gdt, idt};
use x86_64::VirtAddr;
use crate::devices::pic::pic;

pub mod printer;
pub mod interrupts;
pub mod memory;
pub mod task;
pub mod terminal;
pub mod clock;
pub mod logger;
pub mod devices;

pub fn init(rsdp: usize, physical_memory_offset: VirtAddr) {
    print!("\t> Initializing GDT... ");
    gdt::init_gdt();
    println!("initialized!");

    print!("\t> Initializing IDT... ");
    idt::init_idt();
    println!("initialized!");

    print!("\t> Initializing ");
    pic::init_pic(rsdp, physical_memory_offset);
    println!("initialized!");

    print!("\t> Enabling interrupts... ");
    x86_64::instructions::interrupts::enable();
    println!("enabled!");
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}