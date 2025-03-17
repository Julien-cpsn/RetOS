use spin::Mutex;
use talc::{ClaimOnOom, Span, Talc, Talck};

/// 64 MB
pub const HEAP_SIZE: usize = 64 * 1000 * 1000;

pub static mut ARENA: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
pub static ALLOCATOR: Talck<Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
    ClaimOnOom::new(Span::from_array(core::ptr::addr_of!(ARENA).cast_mut()))
}).lock::<Mutex<()>>();