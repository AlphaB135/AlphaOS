//! Paging configuration for x86_64. Currently stubs identity mapping for the lower half.

use crate::arch::transition;
use crate::BootInfo;

/// Configure bootstrap page tables. Replace with full mapper once frame allocator is in place.
pub fn init_identity(_boot_info: &BootInfo) {
    // TODO: Build new page tables using x86_64::structures::paging APIs
    // and install higher-half mappings for the kernel image + framebuffer.
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
