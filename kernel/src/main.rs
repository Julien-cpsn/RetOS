#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use goolog::init_logger;
use goolog::log::{set_max_level, Level, LevelFilter};
use retos_kernel::logger::print_log;
use retos_kernel::memory::tables::{MAPPER, MEMORY_REGIONS};
use retos_kernel::task::executor::{run_tasks, spawn_task};
use retos_kernel::task::keyboard;
use retos_kernel::task::task::Task;
use retos_kernel::terminal::commands::scanpci::scanpci;
use retos_kernel::{memory, printer, println};
use spin::{Mutex, RwLock};
use x86_64::VirtAddr;

const HELLO_WORLD: &str = r#"
/----------------------------------\
|            Welcome to            |
|   _____      _    ____   _____   |
|  |  __ \    | |  / __ \ / ____|  |
|  | |__) |___| |_| |  | | (___    |
|  |  _  // _ \ __| |  | |\___ \   |
|  | | \ \  __/ |_| |__| |____) |  |
|  |_|  \_\___|\__|\____/|_____/   |
\----------------------------------/
"#;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let pre = unsafe { core::arch::x86_64::_rdtsc() };

    /* --- Gathering boot informations --- */
    
    let framebuffer = boot_info.framebuffer.as_mut().expect("No framebuffer");
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset.take().expect("No physical memory"));
    let memory_regions = boot_info.memory_regions.to_vec();
    let rsdp = boot_info.rsdp_addr.take().expect("Failed to get RSDP address") as usize;
    
    /* --- Framebuffer initialization --- */

    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    printer::buffer::set_framebuffer(buffer, info);

    /* --- Hello world --- */

    println!("{HELLO_WORLD}");
    println!();

    /* --- Memory pagination --- */

    MAPPER.call_once(|| RwLock::new(unsafe { memory::tables::init(physical_memory_offset) }));
    MEMORY_REGIONS.call_once(|| Mutex::new(memory_regions));
    
    /* --- Kernel initialization --- */
    
    println!("Initializing kernel...");
    retos_kernel::init(rsdp, physical_memory_offset);
    let post = unsafe { core::arch::x86_64::_rdtsc() };
    println!("Kernel initialized! ({} CPU cycles)", (post - pre) / 100_000);
    println!();

    /* --- Logger initialization --- */
    
    init_logger(
        Some(Level::Trace),
        None,
        &|_timestamp, target, level, args| print_log(target, level, args)
    )
        .expect("Could not initialize logger");

    set_max_level(LevelFilter::Info);
    
    /* --- Kernel loop --- */

    spawn_task(Task::new(String::from("Scan PCI"), async { scanpci().unwrap(); }));
    spawn_task(Task::new(String::from("Terminal"), keyboard::handle_keyboard()));
    run_tasks();
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}