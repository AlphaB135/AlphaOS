//! Intel VT-x initialization stubs.

use core::arch::asm;

#[derive(Debug)]
pub enum VmError {
    Unsupported,
    VmxonFailed,
}

/// Check CPUID for VMX support.
pub fn is_supported() -> bool {
    // TODO: Execute CPUID leaf 1 and query VMX bit.
    true
}

/// SAFETY: caller must ensure CR0/CR4 have required bits set.
pub fn init() {
    unsafe {
        let region: u64 = 0; // Placeholder physical address.
        let ptr = &region as *const u64;
        let res = vmxon(ptr);
        if res.is_err() {
            log::warn!("vmxon failed");
        }
    }
}

unsafe fn vmxon(region: *const u64) -> Result<(), VmError> {
    let mut success: u8 = 1;
    asm!(
        "vmxon [{region}]\n\
         setna {success}",
        region = in(reg) region,
        success = out(reg_byte) success,
        options(nostack)
    );
    if success == 0 { Ok(()) } else { Err(VmError::VmxonFailed) }
}

pub unsafe fn vmxoff() {
    asm!("vmxoff", options(nostack));
}

pub unsafe fn vmwrite(field: u64, value: u64) {
    asm!("vmwrite {value}, {field}", value = in(reg) value, field = in(reg) field, options(nostack));
}

pub unsafe fn vmread(field: u64) -> u64 {
    let mut value = 0u64;
    asm!("vmread {field}, {value}", field = in(reg) field, value = out(reg) value, options(nostack));
    value
}
