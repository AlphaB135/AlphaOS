//! Intel VT-x initialization helpers with inline assembly.

use core::arch::asm;

use crate::msr;

const IA32_FEATURE_CONTROL: u32 = 0x3A;
const IA32_VMX_BASIC: u32 = 0x480;
const CR4_VMXE: u64 = 1 << 13;
const FEATURE_CONTROL_LOCK: u64 = 1;
const FEATURE_CONTROL_VMXON_OUTSIDE_SMX: u64 = 1 << 2;

#[derive(Debug)]
pub enum VmError {
    Unsupported,
    FeatureControlLocked,
    VmxonFailed,
}

#[repr(align(4096))]
struct AlignedRegion {
    bytes: [u8; 4096],
}

static mut VMXON_REGION: AlignedRegion = AlignedRegion { bytes: [0; 4096] };
static mut VMCS_REGION: AlignedRegion = AlignedRegion { bytes: [0; 4096] };
static mut ACTIVE: bool = false;

/// Check CPUID leaf 1 ECX bit 5 for VMX support.
pub fn is_supported() -> bool {
    let (_, _, ecx, _) = cpuid(1, 0);
    ecx & (1 << 5) != 0
}

/// Initialize VMX operation: enable CR4.VMXE, prepare VMXON and VMCS regions, and issue VMXON.
pub fn init() -> Result<(), VmError> {
    if !is_supported() {
        return Err(VmError::Unsupported);
    }

    unsafe {
        enable_feature_control()?;
        enable_cr4_vmx();
        prepare_regions();
        vmxon(VMXON_REGION.bytes.as_ptr())?;
        ACTIVE = true;
    }
    Ok(())
}

/// Expose VMCS region pointer for vCPU setup.
pub unsafe fn vmcs_region() -> Option<*mut u8> {
    if ACTIVE {
        Some(VMCS_REGION.bytes.as_mut_ptr())
    } else {
        None
    }
}

pub fn revision_id() -> u32 {
    unsafe { (msr::rdmsr(IA32_VMX_BASIC) & 0xFFFF_FFFF) as u32 }
}

unsafe fn enable_feature_control() -> Result<(), VmError> {
    let mut value = msr::rdmsr(IA32_FEATURE_CONTROL);
    if value & FEATURE_CONTROL_LOCK == 0 {
        value |= FEATURE_CONTROL_LOCK | FEATURE_CONTROL_VMXON_OUTSIDE_SMX;
        msr::wrmsr(IA32_FEATURE_CONTROL, value);
    } else if value & FEATURE_CONTROL_VMXON_OUTSIDE_SMX == 0 {
        return Err(VmError::FeatureControlLocked);
    }
    Ok(())
}

unsafe fn enable_cr4_vmx() {
    let mut cr4 = msr::read_cr4();
    cr4 |= CR4_VMXE;
    msr::write_cr4(cr4);
}

unsafe fn prepare_regions() {
    let revision = revision_id().to_le_bytes();
    VMXON_REGION.bytes[..4].copy_from_slice(&revision);
    VMCS_REGION.bytes[..4].copy_from_slice(&revision);
}

unsafe fn vmxon(region: *const u8) -> Result<(), VmError> {
    let mut status: u8 = 1;
    asm!(
        "vmxon [{region}]\n\
         setna {status}",
        region = in(reg) region,
        status = out(reg_byte) status,
        options(nostack)
    );
    if status == 0 { Ok(()) } else { Err(VmError::VmxonFailed) }
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

pub fn active() -> bool { unsafe { ACTIVE } }
