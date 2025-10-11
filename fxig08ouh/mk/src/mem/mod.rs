//! Memory subsystem: frame allocation, paging, and heap bootstrap.

pub mod heap;
pub mod paging;

use crate::{BootInfo, MemoryRegionKind};

/// Primary memory initialization entry.
pub fn init(boot_info: &BootInfo) {
    paging::init_identity(boot_info);
    heap::init(boot_info);
}

/// Iterate memory regions that firmware marks as usable.
pub fn usable_regions<'a>(boot_info: &'a BootInfo) -> impl Iterator<Item = &'a crate::MemoryRegion> {
    boot_info
        .memory_regions
        .iter()
        .filter(|region| region.kind == MemoryRegionKind::Usable)
}
