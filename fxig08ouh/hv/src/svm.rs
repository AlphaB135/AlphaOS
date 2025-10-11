//! AMD SVM initialization helpers.

use core::arch::asm;

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

/// Check CPUID vendor string and SVM bit.
pub fn is_supported() -> bool {
    let vendor = vendor_id();
    if vendor != *b"AuthenticAMD" {
        return false;
    }
    let (_, _, ecx, edx) = cpuid(0x8000_0001, 0);
    (ecx & (1 << 2) != 0) || (edx & (1 << 2) != 0)
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
    let rip_ptr = vmcb.add(0x480) as *mut u64;
    let rsp_ptr = vmcb.add(0x488) as *mut u64;
    let rflags_ptr = vmcb.add(0x490) as *mut u64;
    rip_ptr.write(guest_rip);
    rsp_ptr.write(guest_rsp);
    rflags_ptr.write(0x2);
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

pub fn active() -> bool { unsafe { ACTIVE } }

unsafe fn enable_svme() {
    let mut efer = msr::rdmsr(IA32_EFER);
    efer |= EFER_SVME;
    msr::wrmsr(IA32_EFER, efer);
}

fn vendor_id() -> [u8; 12] {
    let (eax, ebx, ecx, edx) = cpuid(0, 0);
    let mut out = [0u8; 12];
    out[0..4].copy_from_slice(&ebx.to_le_bytes());
    out[4..8].copy_from_slice(&edx.to_le_bytes());
    out[8..12].copy_from_slice(&ecx.to_le_bytes());
    out
}

fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let mut eax = leaf;
    let mut ebx: u32;
    let mut ecx = subleaf;
    let mut edx: u32;
    unsafe {
        asm!(
            "cpuid",
            inout("eax") eax,
            out("ebx") ebx,
            inout("ecx") ecx,
            out("edx") edx,
            options(nostack)
        );
    }
    (eax, ebx, ecx, edx)
}
