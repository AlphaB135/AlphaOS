//! UEFI loader helpers: gather memory map, construct initial page tables, and hand off to microkernel.

use alloc::vec::Vec;

use mk::mem::frame::FrameDescriptor;
use mk::mem::paging::BootPageTables;
use mk::{BootInfo, MemoryRegion, MemoryRegionKind};
use uefi::table::boot::{MemoryDescriptor, MemoryType};

use crate::mmap::MAX_MEMORY_REGIONS;

/// Describe page tables allocated during boot to feed into the microkernel.
pub struct PagingInit {
    pub pml4_phys: u64,
    pub frames_used: Vec<FrameDescriptor>,
}

pub fn build_boot_info(
    descriptors: &[MemoryDescriptor],
    framebuffer: Option<microkernel::FrameBufferInfo>,
    rsdp: Option<u64>,
) -> BootInfo {
    let mut regions = mk::BootMemoryMap::new();
    for desc in descriptors.iter().take(MAX_MEMORY_REGIONS) {
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
