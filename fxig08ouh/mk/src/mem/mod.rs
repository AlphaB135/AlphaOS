//! Memory subsystem: frame allocation, paging, and heap bootstrap.

pub mod frame;
pub mod heap;
pub mod paging;

use crate::{BootInfo, MemoryRegionKind};

/// Primary memory initialization entry.
pub fn init(boot_info: &BootInfo) {
    frame::init(boot_info);
    let pml4 = paging::build_kernel_mappings(boot_info);
    unsafe { crate::arch::transition::enable_long_mode(pml4); }
    paging::activate_root(pml4);
    heap::init(boot_info);
}

/// Iterate memory regions that firmware marks as usable.
pub fn usable_regions<'a>(boot_info: &'a BootInfo) -> impl Iterator<Item = &'a crate::MemoryRegion> {
    boot_info
        .memory_regions
        .iter()
        .filter(|region| region.kind == MemoryRegionKind::Usable)
}
