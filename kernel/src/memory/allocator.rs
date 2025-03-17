use crate::memory::tables::MEMORY_REGIONS;
use bootloader_api::info::MemoryRegionKind;
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub static BOOT_INFO_FRAME_ALLOCATOR: Mutex<BootInfoFrameAllocator> = Mutex::new(BootInfoFrameAllocator(0));

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator(usize);

unsafe impl<T: PageSize> FrameAllocator<T> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<T>> {
        let memory_regions = MEMORY_REGIONS.get().unwrap().lock();

        let usable_regions = memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable);

        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.start..r.end);

        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(T::SIZE as usize));

        // create `PhysFrame` types from the start addresses
        let frame = frame_addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
            .nth(self.0);
        
        self.0 += 1;

        frame
    }
}

impl FrameDeallocator<Size4KiB> for BootInfoFrameAllocator {
    unsafe fn deallocate_frame(&mut self, _frame: PhysFrame<Size4KiB>) {

    }
}