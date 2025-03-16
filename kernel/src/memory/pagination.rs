use x86_64::structures::paging::mapper::TranslateError;
use x86_64::structures::paging::{Mapper, Page, Size4KiB};
use x86_64::VirtAddr;


#[allow(dead_code)]
pub fn is_huge_page<T>(virt_addr: VirtAddr, mapper: &T) -> bool where T: Mapper<Size4KiB> {
    let page = Page::<Size4KiB>::containing_address(virt_addr);

    match mapper.translate_page(page) {
        Ok(_) => false,
        Err(e) => match e {
            TranslateError::PageNotMapped => false,
            TranslateError::ParentEntryHugePage => true,
            TranslateError::InvalidFrameAddress(_) => false
        },
    }
}
