use heapless::Vec;
use mk::{MemoryRegion, MemoryRegionKind};
use uefi::table::boot::BootServices;

/// Upper bound on descriptors captured during boot. Tune once real hardware data arrives.
pub const MAX_MEMORY_REGIONS: usize = 256;

/// Collect the firmware memory map and translate it into the microkernel representation.
pub fn collect_memory_map(_bs: &BootServices) -> Vec<MemoryRegion, MAX_MEMORY_REGIONS> {
    // TODO: Use `BootServices::memory_map` to populate `regions` with a faithful descriptor list.
    let mut regions = Vec::<MemoryRegion, MAX_MEMORY_REGIONS>::new();
    let _ = regions.push(MemoryRegion {
        base: 0,
        length: 0,
        kind: MemoryRegionKind::FirmwareReserved,
    });
    regions
}
