//! VM manager skeleton structures with VMX/SVM region setup.

use core::arch::asm;

use hv::{svm, vmx};

#[derive(Default)]
pub struct Vcpu {
    pub id: u16,
    pub vmcs_region: *mut u8,
    pub vmcb_region: *mut u8,
}

static mut VCPU: Vcpu = Vcpu {
    id: 0,
    vmcs_region: core::ptr::null_mut(),
    vmcb_region: core::ptr::null_mut(),
};

pub fn bootstrap() {
    unsafe {
        VCPU = Vcpu::default();
        if vmx::active() {
            if let Some(region) = vmx::vmcs_region() {
                VCPU.vmcs_region = region;
            }
        } else if svm::active() {
            if let Some(vmcb) = svm::vmcb_ptr() {
                VCPU.vmcb_region = vmcb;
            }
        }
    }
}

pub fn vcpu() -> &'static Vcpu {
    unsafe { &VCPU }
}

pub unsafe fn launch() {
    if !VCPU.vmcs_region.is_null() {
        vmx_launch(VCPU.vmcs_region);
    } else if !VCPU.vmcb_region.is_null() {
        svm::vmrun(VCPU.vmcb_region);
    }
}

unsafe fn vmx_launch(vmcs: *mut u8) {
    asm!(
        "vmclear [{vmcs}]\n\
         vmptrld [{vmcs}]\n\
         vmlaunch",
        vmcs = in(reg) vmcs,
        options(nostack)
    );
}
