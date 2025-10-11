//! Paging configuration for x86_64. Builds bootstrap identity mappings and activates long mode.

use crate::arch::transition;
use crate::mem::frame;
use crate::BootInfo;

const ENTRIES: usize = 512;
const PRESENT_WRITE: u64 = 0x3;
const HUGE_PAGE: u64 = 1 << 7;

#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; ENTRIES],
}

/// Build identity-mapped paging structures for early bring-up and return the physical address of the PML4.
pub fn build_kernel_mappings(boot_info: &BootInfo) -> u64 {
    unsafe {
        let pml4_phys = alloc_table();
        let pdp_phys = alloc_table();
        let pd_phys = alloc_table();

        let pml4 = pml4_phys as *mut PageTable;
        let pdp = pdp_phys as *mut PageTable;
        let pd = pd_phys as *mut PageTable;

        zero_table(pml4);
        zero_table(pdp);
        zero_table(pd);

        (*pml4).entries[0] = pdp_phys | PRESENT_WRITE;
        (*pdp).entries[0] = pd_phys | PRESENT_WRITE;

        for i in 0..ENTRIES {
            (*pd).entries[i] = (i as u64 * 0x200000) | PRESENT_WRITE | HUGE_PAGE;
        }

        if let Some(fb) = boot_info.framebuffer {
            let fb_phys = fb.phys_addr & !0x1F_FFFF;
            let index = (fb_phys >> 21) as usize;
            if index < ENTRIES {
                (*pd).entries[index] = fb_phys | PRESENT_WRITE | HUGE_PAGE;
            }
        }

        pml4_phys
    }
}

unsafe fn alloc_table() -> u64 {
    let frame = frame::alloc().expect("out of physical memory during paging setup");
    frame
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
