//! UEFI loader helpers: gather memory map, translate it, and hand off to the microkernel.

use mk::{BootInfo, MemoryRegion, MemoryRegionKind};
use uefi::table::boot::{MemoryMap, MemoryType};

use crate::mmap::MAX_MEMORY_REGIONS;

/// Translate the firmware memory map into the microkernel representation.
pub fn build_boot_info(
    memory_map: &MemoryMap<'static>,
    framebuffer: Option<microkernel::FrameBufferInfo>,
    rsdp: Option<u64>,
) -> BootInfo {
    let mut regions = mk::BootMemoryMap::new();
    for desc in memory_map.entries().take(MAX_MEMORY_REGIONS) {
        let region = MemoryRegion {
            base: desc.phys_start,
            length: desc.page_count * 4096,
            kind: translate_memory_type(desc.ty),
        };
        let _ = regions.push(region);
    }

    BootInfo {
        memory_regions: regions,
        framebuffer,
        rsdp_addr: rsdp,
    }
}

fn translate_memory_type(ty: MemoryType) -> MemoryRegionKind {
    match ty {
        MemoryType::CONVENTIONAL => MemoryRegionKind::Usable,
        MemoryType::BOOT_SERVICES_CODE
        | MemoryType::BOOT_SERVICES_DATA
        | MemoryType::RUNTIME_SERVICES_CODE
        | MemoryType::RUNTIME_SERVICES_DATA => MemoryRegionKind::Reserved,
        MemoryType::ACPI_RECLAIM => MemoryRegionKind::AcpiReclaimable,
        MemoryType::ACPI_NON_VOLATILE => MemoryRegionKind::AcpiNvs,
        MemoryType::MMIO => MemoryRegionKind::Mmio,
        _ => MemoryRegionKind::FirmwareReserved,
    }
}
