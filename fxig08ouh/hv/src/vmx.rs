//! Intel VT-x initialization helpers with inline assembly.

use core::arch::asm;

use crate::msr;

const IA32_FEATURE_CONTROL: u32 = 0x3A;
const IA32_VMX_BASIC: u32 = 0x480;
const IA32_VMX_PINBASED_CTLS: u32 = 0x481;
const IA32_VMX_PROCBASED_CTLS: u32 = 0x482;
const IA32_VMX_EXIT_CTLS: u32 = 0x483;
const IA32_VMX_ENTRY_CTLS: u32 = 0x484;

const CR4_VMXE: u64 = 1 << 13;
const FEATURE_CONTROL_LOCK: u64 = 1;
const FEATURE_CONTROL_VMXON_OUTSIDE_SMX: u64 = 1 << 2;

pub const VMCS_EXIT_REASON: u64 = 0x4402;
const VMCS_PINBASED_CTLS: u64 = 0x4000;
const VMCS_CPU_BASED_CTLS: u64 = 0x4002;
const VMCS_EXIT_CTLS_FIELD: u64 = 0x400C;
const VMCS_ENTRY_CTLS_FIELD: u64 = 0x4012;
const VMCS_HOST_CR0: u64 = 0x6C00;
const VMCS_HOST_CR3: u64 = 0x6C02;
const VMCS_HOST_CR4: u64 = 0x6C04;
const VMCS_HOST_RSP: u64 = 0x6C14;
const VMCS_HOST_RIP: u64 = 0x6C16;
const VMCS_HOST_CS_SELECTOR: u64 = 0x0C02;
const VMCS_HOST_SS_SELECTOR: u64 = 0x0C04;
const VMCS_HOST_DS_SELECTOR: u64 = 0x0C06;
const VMCS_HOST_ES_SELECTOR: u64 = 0x0C00;
const VMCS_HOST_FS_SELECTOR: u64 = 0x0C08;
const VMCS_HOST_GS_SELECTOR: u64 = 0x0C0A;
const VMCS_HOST_TR_SELECTOR: u64 = 0x0C0C;
const VMCS_GUEST_CR0: u64 = 0x6800;
const VMCS_GUEST_CR3: u64 = 0x6802;
const VMCS_GUEST_CR4: u64 = 0x6804;
const VMCS_GUEST_RSP: u64 = 0x681C;
const VMCS_GUEST_RIP: u64 = 0x681E;
const VMCS_GUEST_RFLAGS: u64 = 0x6820;
const VMCS_GUEST_CS_SELECTOR: u64 = 0x0802;
const VMCS_GUEST_SS_SELECTOR: u64 = 0x0804;
const VMCS_GUEST_DS_SELECTOR: u64 = 0x0806;
const VMCS_GUEST_ES_SELECTOR: u64 = 0x0800;
const VMCS_GUEST_FS_SELECTOR: u64 = 0x0808;
const VMCS_GUEST_GS_SELECTOR: u64 = 0x080A;
const VMCS_GUEST_TR_SELECTOR: u64 = 0x080C;

#[derive(Debug)]
pub enum VmError {
    Unsupported,
    FeatureControlLocked,
    VmxonFailed,
    InstructionFailure(&'static str),
}

#[repr(align(4096))]
struct AlignedRegion {
    bytes: [u8; 4096],
}

static mut VMXON_REGION: AlignedRegion = AlignedRegion { bytes: [0; 4096] };
static mut VMCS_REGION: AlignedRegion = AlignedRegion { bytes: [0; 4096] };
static mut ACTIVE: bool = false;

pub struct HostState {
    pub cr0: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub rip: u64,
    pub rsp: u64,
}

pub struct GuestState {
    pub cr0: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub rip: u64,
    pub rsp: u64,
}

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

pub fn active() -> bool { unsafe { ACTIVE } }

pub unsafe fn load_vmcs(region: *mut u8) -> Result<(), VmError> {
    vmclear(region as *const u8)?;
    vmptrld(region as *const u8)
}

pub unsafe fn configure_vmcs(host: &HostState, guest: &GuestState) -> Result<(), VmError> {
    let pin = adjust_controls(IA32_VMX_PINBASED_CTLS, 0);
    let cpu = adjust_controls(IA32_VMX_PROCBASED_CTLS, 0);
    let exit = adjust_controls(IA32_VMX_EXIT_CTLS, 0);
    let entry = adjust_controls(IA32_VMX_ENTRY_CTLS, 0);

    vmwrite(VMCS_PINBASED_CTLS, pin as u64)?;
    vmwrite(VMCS_CPU_BASED_CTLS, cpu as u64)?;
    vmwrite(VMCS_EXIT_CTLS_FIELD, exit as u64)?;
    vmwrite(VMCS_ENTRY_CTLS_FIELD, entry as u64)?;

    vmwrite(VMCS_HOST_CR0, host.cr0)?;
    vmwrite(VMCS_HOST_CR3, host.cr3)?;
    vmwrite(VMCS_HOST_CR4, host.cr4)?;
    vmwrite(VMCS_HOST_RSP, host.rsp)?;
    vmwrite(VMCS_HOST_RIP, host.rip)?;
    vmwrite(VMCS_HOST_CS_SELECTOR, 0x08)?;
    vmwrite(VMCS_HOST_SS_SELECTOR, 0x10)?;
    vmwrite(VMCS_HOST_DS_SELECTOR, 0x10)?;
    vmwrite(VMCS_HOST_ES_SELECTOR, 0x10)?;
    vmwrite(VMCS_HOST_FS_SELECTOR, 0)?;
    vmwrite(VMCS_HOST_GS_SELECTOR, 0)?;
    vmwrite(VMCS_HOST_TR_SELECTOR, 0)?;

    vmwrite(VMCS_GUEST_CR0, guest.cr0)?;
    vmwrite(VMCS_GUEST_CR3, guest.cr3)?;
    vmwrite(VMCS_GUEST_CR4, guest.cr4)?;
    vmwrite(VMCS_GUEST_RSP, guest.rsp)?;
    vmwrite(VMCS_GUEST_RIP, guest.rip)?;
    vmwrite(VMCS_GUEST_RFLAGS, 0x2)?;
    vmwrite(VMCS_GUEST_CS_SELECTOR, 0x08)?;
    vmwrite(VMCS_GUEST_SS_SELECTOR, 0x10)?;
    vmwrite(VMCS_GUEST_DS_SELECTOR, 0x10)?;
    vmwrite(VMCS_GUEST_ES_SELECTOR, 0x10)?;
    vmwrite(VMCS_GUEST_FS_SELECTOR, 0)?;
    vmwrite(VMCS_GUEST_GS_SELECTOR, 0)?;
    vmwrite(VMCS_GUEST_TR_SELECTOR, 0)?;

    Ok(())
}

pub unsafe fn vmwrite(field: u64, value: u64) -> Result<(), VmError> {
    let mut status: u8;
    asm!(
        "vmwrite {value}, {field}\n\
         setna {status}",
        value = in(reg) value,
        field = in(reg) field,
        status = out(reg_byte) status,
        options(nostack)
    );
    if status == 0 { Ok(()) } else { Err(VmError::InstructionFailure("vmwrite")) }
}

pub unsafe fn vmread(field: u64) -> Result<u64, VmError> {
    let mut value = 0u64;
    let mut status: u8;
    asm!(
        "vmread {field}, {value}\n\
         setna {status}",
        field = in(reg) field,
        value = out(reg) value,
        status = out(reg_byte) status,
        options(nostack)
    );
    if status == 0 { Ok(value) } else { Err(VmError::InstructionFailure("vmread")) }
}

unsafe fn vmclear(region: *const u8) -> Result<(), VmError> {
    let mut status: u8;
    asm!(
        "vmclear [{region}]\n\
         setna {status}",
        region = in(reg) region,
        status = out(reg_byte) status,
        options(nostack)
    );
    if status == 0 { Ok(()) } else { Err(VmError::InstructionFailure("vmclear")) }
}

unsafe fn vmptrld(region: *const u8) -> Result<(), VmError> {
    let mut status: u8;
    asm!(
        "vmptrld [{region}]\n\
         setna {status}",
        region = in(reg) region,
        status = out(reg_byte) status,
        options(nostack)
    );
    if status == 0 { Ok(()) } else { Err(VmError::InstructionFailure("vmptrld")) }
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

fn adjust_controls(msr_id: u32, requested: u32) -> u32 {
    unsafe {
        let value = msr::rdmsr(msr_id);
        let allowed0 = value as u32;
        let allowed1 = (value >> 32) as u32;
        (requested | allowed0) & allowed1
    }
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
