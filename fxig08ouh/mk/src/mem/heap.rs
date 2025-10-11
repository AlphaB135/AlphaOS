//! Heap bootstrap driven by the firmware memory map.

use core::cmp::min;

use linked_list_allocator::LockedHeap;
use spin::Once;

use crate::BootInfo;

const MAX_HEAP_BYTES: usize = 16 * 1024 * 1024; // 16 MiB temporary heap until physical allocator lands.

static HEAP_ALLOCATOR: Once<&'static LockedHeap> = Once::new();

/// Register the global allocator instance declared in `lib.rs`.
pub unsafe fn bootstrap_global_heap(heap: &'static LockedHeap) {
    HEAP_ALLOCATOR.call_once(|| heap);
}

/// Initialize the heap with the first usable region from boot info.
pub fn init(boot_info: &BootInfo) {
    if let Some(heap) = HEAP_ALLOCATOR.get() {
        if let Some(region) = boot_info
            .memory_regions
            .iter()
            .find(|descriptor| descriptor.kind == crate::MemoryRegionKind::Usable)
        {
            let start = region.base as usize;
            let size = min(region.length as usize, MAX_HEAP_BYTES);
            unsafe {
                // SAFETY: start/size derived from firmware-provided usable region.
                heap.lock().init(start as *mut u8, size);
            }
        }
    }
}
