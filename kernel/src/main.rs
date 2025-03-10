#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use goolog::init_logger;
use goolog::log::Level;
use retos_kernel::clock::MilliSecondClock;
use retos_kernel::logger::print_log;
use retos_kernel::task::executor::{run_tasks, spawn_task};
use retos_kernel::task::keyboard;
use retos_kernel::task::task::Task;
use retos_kernel::{printer, println};

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

    /* --- Framebuffer initialization --- */

    let framebuffer = boot_info.framebuffer.as_mut().expect("No framebuffer");
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    printer::buffer::set_framebuffer(buffer, info);

    /* --------------------------------- */

    println!("{HELLO_WORLD}");
    println!();

    /* --- Kernel initialization --- */

    println!("Initializing kernel...");
    retos_kernel::init();
    let post = unsafe { core::arch::x86_64::_rdtsc() };
    println!(
        "Kernel initialized! ({} CPU cycles)",
        (post - pre) / 100_000
    );
    println!();

    /* --- Logger initialization --- */

    init_logger(
        Some(Level::Trace),
        None,
        &|_timestamp, target, level, args| {
            let timestamp = MilliSecondClock::format();
            print_log(&timestamp, target, level, args);
        },
    )
    .expect("Could not initialize logger");

    /* --------------------------------- */

    // Paginate memory
    /*
    let physical_memory_offset = boot_info.physical_memory_offset.take().expect("No physical memory");
    let mut mapper = unsafe { memory::tables::init(VirtAddr::new(physical_memory_offset)) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    */

    println!("======= User input starts =======");

    /* --- Kernel loop --- */

    spawn_task(Task::new(
        String::from("Terminal"),
        keyboard::handle_keyboard(),
    ));
    run_tasks();
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

