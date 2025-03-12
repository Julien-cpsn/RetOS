use spin::Mutex;
use talc::{ClaimOnOom, Span, Talc, Talck};

pub const HEAP_START: usize = 0x_4444_4444_0000;
/// 32 MB
pub const HEAP_SIZE: usize = 32 * 1000 * 1000;

pub static mut ARENA: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Init the heap allocator
pub fn init_allocator() {
    #[global_allocator]
    static TALCK: Talck<Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
        ClaimOnOom::new(Span::from_array(core::ptr::addr_of!(ARENA).cast_mut()))
    }).lock::<Mutex<()>>();
    
    unsafe {
        TALCK.lock().claim(Span::from(&raw mut ARENA)).expect("Could not claim heap");
    }
}