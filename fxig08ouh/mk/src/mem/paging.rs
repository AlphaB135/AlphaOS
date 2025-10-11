//! Paging configuration for x86_64. Currently stubs identity mapping for the lower half.

use core::ptr::addr_of_mut;

use crate::arch::transition;
use crate::BootInfo;

const ENTRIES: usize = 512;

#[repr(align(4096))]
struct PageTable {
    entries: [u64; ENTRIES],
}

static mut PML4: PageTable = PageTable { entries: [0; ENTRIES] };
static mut PDP: PageTable = PageTable { entries: [0; ENTRIES] };
static mut PD: PageTable = PageTable { entries: [0; ENTRIES] };

const PRESENT_WRITE: u64 = 0x3;
const HUGE_PAGE: u64 = 1 << 7;

pub fn build_kernel_mappings(boot_info: &BootInfo) -> u64 {
    unsafe {
        clear_tables();
        let pml4 = addr_of_mut!(PML4) as *mut PageTable;
        let pdp = addr_of_mut!(PDP) as *mut PageTable;
        let pd = addr_of_mut!(PD) as *mut PageTable;

        (*pml4).entries[0] = (pdp as u64) | PRESENT_WRITE;
        (*pdp).entries[0] = (pd as u64) | PRESENT_WRITE;

        for i in 0..ENTRIES {
            (*pd).entries[i] = (i as u64 * 0x200000) | PRESENT_WRITE | HUGE_PAGE;
        }

        let fb_phys = boot_info
            .framebuffer
            .map(|fb| fb.phys_addr & !0xFFF)
            .unwrap_or(0);
        if fb_phys != 0 {
            let index = (fb_phys >> 21) as usize;
            if index < ENTRIES {
                (*pd).entries[index] = fb_phys | PRESENT_WRITE | HUGE_PAGE;
            }
        }

        pml4 as u64
    }
}

unsafe fn clear_tables() {
    PML4.entries.fill(0);
    PDP.entries.fill(0);
    PD.entries.fill(0);
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
