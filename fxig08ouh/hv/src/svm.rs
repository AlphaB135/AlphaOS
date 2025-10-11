//! AMD SVM initialization helpers.

use core::arch::asm;

use raw_cpuid::CpuId;

use crate::msr;

const IA32_EFER: u32 = 0xC000_0080;
const EFER_SVME: u64 = 1 << 12;

#[derive(Debug)]
pub enum SvmError {
    Unsupported,
}

#[repr(align(4096))]
struct Vmcb {
    bytes: [u8; 4096],
}

static mut VMCB: Vmcb = Vmcb { bytes: [0; 4096] };
static mut ACTIVE: bool = false;

const INTERCEPT_CTRL1: usize = 0x000;
const INTERCEPT_CTRL2: usize = 0x004;
const INTERCEPT_CR_READ: usize = 0x008;
const INTERCEPT_CR_WRITE: usize = 0x00C;
const INTERCEPT_DR_READ: usize = 0x010;
const INTERCEPT_DR_WRITE: usize = 0x014;
const INTERCEPT_EXCEPTION: usize = 0x018;
const INTERCEPT_IOPM_BASE: usize = 0x0D0;
const INTERCEPT_MSRPM_BASE: usize = 0x0D8;

/// Check CPUID vendor string and SVM bit.
pub fn is_supported() -> bool {
    CpuId::new()
        .get_extended_processor_and_feature_identifiers()
        .map_or(false, |info| info.has_svm())
}

pub fn init() -> Result<(), SvmError> {
    if !is_supported() {
        return Err(SvmError::Unsupported);
    }
    unsafe {
        enable_svme();
        ACTIVE = true;
    }
    Ok(())
}

pub unsafe fn vmcb_ptr() -> Option<*mut u8> {
    if ACTIVE {
        Some(VMCB.bytes.as_mut_ptr())
    } else {
        None
    }
}

pub unsafe fn configure_vmcb(vmcb: *mut u8, guest_rip: u64, guest_rsp: u64) {
    core::ptr::write_bytes(vmcb, 0, 4096);
    let ctrl1 = vmcb.add(INTERCEPT_CTRL1) as *mut u32;
    let ctrl2 = vmcb.add(INTERCEPT_CTRL2) as *mut u32;
    ctrl1.write((1 << 0) | (1 << 2)); // intercept hlt + io instructions
    ctrl2.write(0);
    (vmcb.add(INTERCEPT_CR_READ) as *mut u32).write(0);
    (vmcb.add(INTERCEPT_CR_WRITE) as *mut u32).write(0);
    (vmcb.add(INTERCEPT_EXCEPTION) as *mut u32).write(0);
    let rip_ptr = vmcb.add(0x480) as *mut u64;
    let rsp_ptr = vmcb.add(0x488) as *mut u64;
    let rflags_ptr = vmcb.add(0x490) as *mut u64;
    rip_ptr.write(guest_rip);
    rsp_ptr.write(guest_rsp);
    rflags_ptr.write(0x2);
}

pub unsafe fn vmrun(vmcb: *const u8) {
    asm!("vmrun rax", in("rax") vmcb as u64, options(nostack));
}

pub unsafe fn vmsave(vmcb: *const u8) {
    asm!("vmsave rax", in("rax") vmcb as u64, options(nostack));
}

pub unsafe fn vmload(vmcb: *const u8) {
    asm!("vmload rax", in("rax") vmcb as u64, options(nostack));
}

pub fn active() -> bool { unsafe { ACTIVE } }

unsafe fn enable_svme() {
    let mut efer = msr::rdmsr(IA32_EFER);
    efer |= EFER_SVME;
    msr::wrmsr(IA32_EFER, efer);
}
