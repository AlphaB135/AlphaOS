//! Paging configuration for x86_64. Currently stubs identity mapping for the lower half.

use crate::BootInfo;

/// Configure bootstrap page tables. Replace with full mapper once frame allocator is in place.
pub fn init_identity(_boot_info: &BootInfo) {
    // TODO: Build new page tables using x86_64::structures::paging APIs
    // and install higher-half mappings for the kernel image + framebuffer.
}
