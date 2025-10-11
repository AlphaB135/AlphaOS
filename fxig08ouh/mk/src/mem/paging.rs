//! Paging configuration for x86_64. Builds bootstrap identity mappings and activates long mode.

use crate::arch::transition;
use crate::mem::frame;
use crate::BootInfo;

const ENTRIES: usize = 512;
const PRESENT_WRITE: u64 = 0x3;
const HUGE_PAGE: u64 = 1 << 7;
const HIGHER_HALF_BASE: u64 = 0xFFFF_8000_0000_0000;
const MAX_PD_COUNT: usize = 64;

#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; ENTRIES],
}

/// Build identity + higher-half mappings and return the physical address of the PML4.
pub fn build_kernel_mappings(boot_info: &BootInfo) -> u64 {
    unsafe {
        let max_phys = boot_info
            .memory_regions
            .iter()
            .map(|region| region.base + region.length)
            .max()
            .unwrap_or(0);
        let fb_phys = boot_info
            .framebuffer
            .map(|fb| fb.phys_addr + fb.size as u64)
            .unwrap_or(0);
        let max_phys = core::cmp::max(max_phys, fb_phys);
        let total_entries = core::cmp::max(1, ((max_phys + 0x1F_FFFF) / 0x200000) as usize);
        let pd_required = core::cmp::max(1, core::cmp::min(MAX_PD_COUNT, (total_entries + ENTRIES - 1) / ENTRIES));

        let pml4_phys = alloc_table();
        let pdp_phys = alloc_table();
        let pml4 = pml4_phys as *mut PageTable;
        let pdp = pdp_phys as *mut PageTable;
        zero_table(pml4);
        zero_table(pdp);

        let mut pd_phys = [0u64; MAX_PD_COUNT];
        let mut pd_tables: [*mut PageTable; MAX_PD_COUNT] = [core::ptr::null_mut(); MAX_PD_COUNT];
        for i in 0..pd_required {
            pd_phys[i] = alloc_table();
            pd_tables[i] = pd_phys[i] as *mut PageTable;
            zero_table(pd_tables[i]);
            (*pdp).entries[i] = pd_phys[i] | PRESENT_WRITE;
        }

        for entry in 0..core::cmp::min(total_entries, pd_required * ENTRIES) {
            let pd_idx = entry / ENTRIES;
            let pt_idx = entry % ENTRIES;
            let phys = entry as u64 * 0x200000;
            (*pd_tables[pd_idx]).entries[pt_idx] = phys | PRESENT_WRITE | HUGE_PAGE;
        }

        if let Some(fb) = boot_info.framebuffer {
            let fb_phys = fb.phys_addr & !0x1F_FFFF;
            let entry = (fb_phys / 0x200000) as usize;
            if entry < pd_required * ENTRIES {
                let pd_idx = entry / ENTRIES;
                let pt_idx = entry % ENTRIES;
                (*pd_tables[pd_idx]).entries[pt_idx] = fb_phys | PRESENT_WRITE | HUGE_PAGE;
            }
        }

        (*pml4).entries[0] = pdp_phys | PRESENT_WRITE;

        let higher_index = ((HIGHER_HALF_BASE >> 39) & 0x1FF) as usize;
        let pdp_high_phys = alloc_table();
        let pdp_high = pdp_high_phys as *mut PageTable;
        zero_table(pdp_high);
        (*pml4).entries[higher_index] = pdp_high_phys | PRESENT_WRITE;
        for i in 0..pd_required {
            (*pdp_high).entries[i] = pd_phys[i] | PRESENT_WRITE;
        }

        pml4_phys
    }
}

unsafe fn alloc_table() -> u64 {
    frame::alloc().expect("out of physical memory during paging setup")
}

unsafe fn zero_table(table: *mut PageTable) {
    core::ptr::write_bytes(table as *mut u8, 0, core::mem::size_of::<PageTable>());
}

/// Install the physical address of the root page table into CR3.
pub fn activate_root(pml4_phys: u64) {
    unsafe {
        transition::write_cr3(pml4_phys);
        transition::flush_tlb();
    }
}

/// Invalidate a particular page mapping.
pub fn invalidate(addr: u64) {
    unsafe {
        transition::invlpg(addr);
    }
}
