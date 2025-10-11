//! Intel VT-x initialization helpers with inline assembly, including segment/MSR setup for the VMCS.

use core::arch::asm;

use raw_cpuid::CpuId;

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
const VMCS_HOST_FS_BASE: u64 = 0x6C06;
const VMCS_HOST_GS_BASE: u64 = 0x6C08;
const VMCS_HOST_TR_BASE: u64 = 0x6C0A;
const VMCS_HOST_GDTR_BASE: u64 = 0x6C0C;
const VMCS_HOST_IDTR_BASE: u64 = 0x6C0E;
const VMCS_HOST_SYSENTER_CS: u64 = 0x4C00;
const VMCS_HOST_SYSENTER_ESP: u64 = 0x6C10;
const VMCS_HOST_SYSENTER_EIP: u64 = 0x6C12;
const VMCS_HOST_PAT: u64 = 0x2C00;

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
const VMCS_GUEST_LDTR_SELECTOR: u64 = 0x080E;
const VMCS_GUEST_CS_LIMIT: u64 = 0x4802;
const VMCS_GUEST_SS_LIMIT: u64 = 0x4804;
const VMCS_GUEST_DS_LIMIT: u64 = 0x4806;
const VMCS_GUEST_ES_LIMIT: u64 = 0x4800;
const VMCS_GUEST_FS_LIMIT: u64 = 0x4808;
const VMCS_GUEST_GS_LIMIT: u64 = 0x480A;
const VMCS_GUEST_TR_LIMIT: u64 = 0x480C;
const VMCS_GUEST_GDTR_LIMIT: u64 = 0x4810;
const VMCS_GUEST_IDTR_LIMIT: u64 = 0x4812;
const VMCS_GUEST_CS_ACCESS: u64 = 0x4816;
const VMCS_GUEST_SS_ACCESS: u64 = 0x4818;
const VMCS_GUEST_DS_ACCESS: u64 = 0x481A;
const VMCS_GUEST_ES_ACCESS: u64 = 0x4814;
const VMCS_GUEST_FS_ACCESS: u64 = 0x481C;
const VMCS_GUEST_GS_ACCESS: u64 = 0x481E;
const VMCS_GUEST_TR_ACCESS: u64 = 0x4820;
const VMCS_GUEST_LDTR_ACCESS: u64 = 0x4822;
const VMCS_GUEST_FS_BASE: u64 = 0x6806;
const VMCS_GUEST_GS_BASE: u64 = 0x6808;
const VMCS_GUEST_TR_BASE: u64 = 0x680A;
const VMCS_GUEST_GDTR_BASE: u64 = 0x680C;
const VMCS_GUEST_IDTR_BASE: u64 = 0x680E;
const VMCS_GUEST_SYSENTER_CS: u64 = 0x482A;
const VMCS_GUEST_SYSENTER_ESP: u64 = 0x6824;
const VMCS_GUEST_SYSENTER_EIP: u64 = 0x6826;
const VMCS_GUEST_LINK_POINTER: u64 = 0x2800;
const VMCS_GUEST_ACTIVITY_STATE: u64 = 0x4826;
const VMCS_GUEST_INTERRUPTIBILITY: u64 = 0x4824;

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
    pub gdtr_base: u64,
    pub idtr_base: u64,
    pub sysenter_cs: u64,
    pub sysenter_rip: u64,
    pub sysenter_rsp: u64,
    pub fs_base: u64,
    pub gs_base: u64,
}

pub struct GuestState {
    pub cr0: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub rip: u64,
    pub rsp: u64,
    pub gdtr_base: u64,
    pub idtr_base: u64,
    pub fs_base: u64,
    pub gs_base: u64,
}

/// Check CPUID leaf 1 ECX bit 5 for VMX support.
pub fn is_supported() -> bool {
    CpuId::new()
        .get_feature_info()
        .map_or(false, |info| info.has_vmx())
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
    vmwrite(VMCS_HOST_FS_BASE, host.fs_base)?;
    vmwrite(VMCS_HOST_GS_BASE, host.gs_base)?;
    vmwrite(VMCS_HOST_TR_BASE, 0)?;
    vmwrite(VMCS_HOST_GDTR_BASE, host.gdtr_base)?;
    vmwrite(VMCS_HOST_IDTR_BASE, host.idtr_base)?;
    vmwrite(VMCS_HOST_SYSENTER_CS, host.sysenter_cs)?;
    vmwrite(VMCS_HOST_SYSENTER_ESP, host.sysenter_rsp)?;
    vmwrite(VMCS_HOST_SYSENTER_EIP, host.sysenter_rip)?;
    vmwrite(VMCS_HOST_PAT, 0)?;

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
    vmwrite(VMCS_GUEST_LDTR_SELECTOR, 0)?;
    vmwrite(VMCS_GUEST_CS_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_SS_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_DS_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_ES_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_FS_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_GS_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_TR_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_GDTR_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_IDTR_LIMIT, 0xFFFF)?;
    vmwrite(VMCS_GUEST_CS_ACCESS, 0xC09B)?;
    vmwrite(VMCS_GUEST_SS_ACCESS, 0xC093)?;
    vmwrite(VMCS_GUEST_DS_ACCESS, 0xC093)?;
    vmwrite(VMCS_GUEST_ES_ACCESS, 0xC093)?;
    vmwrite(VMCS_GUEST_FS_ACCESS, 0xC093)?;
    vmwrite(VMCS_GUEST_GS_ACCESS, 0xC093)?;
    vmwrite(VMCS_GUEST_TR_ACCESS, 0x808B)?;
    vmwrite(VMCS_GUEST_LDTR_ACCESS, 0x8202)?;
    vmwrite(VMCS_GUEST_FS_BASE, guest.fs_base)?;
    vmwrite(VMCS_GUEST_GS_BASE, guest.gs_base)?;
    vmwrite(VMCS_GUEST_TR_BASE, 0)?;
    vmwrite(VMCS_GUEST_GDTR_BASE, guest.gdtr_base)?;
    vmwrite(VMCS_GUEST_IDTR_BASE, guest.idtr_base)?;
    vmwrite(VMCS_GUEST_SYSENTER_CS, 0)?;
    vmwrite(VMCS_GUEST_SYSENTER_ESP, guest.rsp)?;
    vmwrite(VMCS_GUEST_SYSENTER_EIP, guest.rip)?;
    vmwrite(VMCS_GUEST_LINK_POINTER, 0xFFFF_FFFF_FFFF_FFFF)?;
    vmwrite(VMCS_GUEST_ACTIVITY_STATE, 0)?;
    vmwrite(VMCS_GUEST_INTERRUPTIBILITY, 0)?;

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
