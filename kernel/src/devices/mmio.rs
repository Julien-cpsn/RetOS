use core::num::NonZeroUsize;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use crate::memory::allocator::BOOT_INFO_FRAME_ALLOCATOR;
use crate::memory::tables::MAPPER;


static MMIO_REGION_START: VirtAddr = VirtAddr::new(0xF000_0000);
static MMIO_CURRENT_OFFSET: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct MemoryMapper;

impl accessor::Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        // Allocate space in the MMIO region
        let offset = MMIO_CURRENT_OFFSET.fetch_add(bytes, Ordering::SeqCst);
        let virt_addr = MMIO_REGION_START + offset as u64;
        
        let phys_addr = PhysAddr::new(phys_base as u64);

        let start_frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys_addr);
        let end_frame = PhysFrame::containing_address(phys_addr + bytes as u64 - 1);
        
        let mut current_virt = virt_addr;

        let mut mapper = MAPPER.get().unwrap().write();
        let mut frame_allocator = BOOT_INFO_FRAME_ALLOCATOR.lock();
        
        for frame in PhysFrame::range(start_frame, end_frame) {
            let page = Page::containing_address(current_virt);
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE | PageTableFlags::WRITE_THROUGH;

            let map_result = mapper.map_to(page, frame, flags, frame_allocator.deref_mut());
            map_result.expect("Memory mapping failed").flush();

            current_virt += Size4KiB::SIZE;
        }

        NonZeroUsize::new(virt_addr.as_u64() as usize).expect("Virtual address cannot be zero")
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {
        let virt_addr = VirtAddr::new(virt_base as u64);
        
        let start_page: Page<Size4KiB> = Page::containing_address(virt_addr);
        let end_page = Page::containing_address(virt_addr + bytes as u64 - 1);

        let mut mapper = MAPPER.get().unwrap().write();

        for page in Page::range(start_page, end_page) {
            let res = mapper.unmap(page);
            res.expect("Unmapping failed").1.flush();
        }
    }
}