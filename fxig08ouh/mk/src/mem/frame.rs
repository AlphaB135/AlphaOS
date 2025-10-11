//! Simple frame allocator backed by the firmware memory map.

use core::ops::RangeInclusive;

use crate::{BootInfo, MemoryRegionKind};

static mut NEXT_FRAME: u64 = 0;
static mut LIMIT: u64 = 0;

pub fn init(boot_info: &BootInfo) {
    unsafe {
        for descriptor in boot_info.memory_regions.iter() {
            if descriptor.kind == MemoryRegionKind::Usable {
                NEXT_FRAME = align_up(descriptor.base, 0x1000);
                LIMIT = (descriptor.base + descriptor.length) & !0xFFF;
                break;
            }
        }
    }
}

pub fn alloc() -> Option<u64> {
    unsafe {
        if NEXT_FRAME == 0 || NEXT_FRAME >= LIMIT {
            return None;
        }
        let frame = NEXT_FRAME;
        NEXT_FRAME = NEXT_FRAME.saturating_add(0x1000);
        Some(frame)
    }
}

pub fn range() -> RangeInclusive<u64> {
    unsafe { NEXT_FRAME..=LIMIT }
}

const fn align_up(value: u64, align: u64) -> u64 {
    ((value + align - 1) / align) * align
}
