//! AMD SVM initialization stubs.

use core::arch::asm;

/// Check CPUID vendor string for "AuthenticAMD" and SVM bit.
pub fn is_supported() -> bool {
    false
}

/// Initialize VMCB structures.
pub fn init() {
    // TODO: allocate VMCB and set intercept vectors.
}

pub unsafe fn vmrun(vmcb: *const u8) {
    asm!("vmrun {0}", in(reg) vmcb, options(nostack));
}

pub unsafe fn vmsave(vmcb: *const u8) {
    asm!("vmsave {0}", in(reg) vmcb, options(nostack));
}

pub unsafe fn vmload(vmcb: *const u8) {
    asm!("vmload {0}", in(reg) vmcb, options(nostack));
}
