use alloc::format;
use alloc::string::{String, ToString};
use goolog::{debug, set_target};
use crate::memory::heap_allocator::ALLOCATOR;
use crate::printer::buffer::WRITER;
use crate::terminal::error::CliError;

pub fn top() -> Result<(), CliError> {
    set_target!("TOP");

    debug!("Locking ALLOCATOR mutex...");
    let allocator = ALLOCATOR.lock();
    let heap_counters = *allocator.get_counters();
    drop(allocator);
    debug!("ALLOCATOR mutex freed");


    let ram_size = heap_counters.allocated_bytes / 1000;
    let ram_percentage = heap_counters.total_allocated_bytes as f32 / heap_counters.allocated_bytes as f32;
    
    let table = [
        [String::from("CPU"), String::from("RAM (%)"), String::from("RAM (KiB)")],
        [String::from("TODO"), format!("{:.2}", ram_percentage), ram_size.to_string()],
    ];

    let mut writer = WRITER.write();
    text_tables::render(&mut *writer, table).unwrap();


    Ok(())
}