//! VM manager skeleton structures with VMX/SVM region setup.

use core::arch::asm;

use hv::svm;
use hv::vmx::{self, GuestState, HostState, VMCS_EXIT_REASON};
use log::warn;
use mk::arch::transition;

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

#[repr(align(16))]
static mut HOST_STACK: [u8; 4096] = [0; 4096];
#[repr(align(16))]
static mut GUEST_STACK: [u8; 4096] = [0; 4096];

extern "C" fn vmexit_stub() -> ! {
    unsafe {
        loop {
            asm!("hlt", options(nomem, preserves_flags));
        }
    }
}

extern "C" fn guest_entry_stub() -> ! {
    unsafe {
        loop {
            asm!("hlt", options(nomem, preserves_flags));
        }
    }
}

pub fn bootstrap() {
    unsafe {
        VCPU = Vcpu::default();
        if vmx::active() {
            if let Some(region) = vmx::vmcs_region() {
                if vmx::load_vmcs(region).is_ok() {
                    let host = host_state();
                    let guest = guest_state();
                    if let Err(err) = vmx::configure_vmcs(&host, &guest) {
                        warn!("VMX configure failed: {:?}", err);
                    } else {
                        VCPU.vmcs_region = region;
                    }
                }
            }
        } else if svm::active() {
            if let Some(vmcb) = svm::vmcb_ptr() {
                svm::configure_vmcb(vmcb, guest_entry_stub as u64, guest_stack_top());
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
        svm_launch(VCPU.vmcb_region);
    }
}

unsafe fn vmx_launch(_vmcs: *mut u8) {
    loop {
        let mut status: u8;
        asm!(
            "vmlaunch\n\
             setna {status}",
            status = out(reg_byte) status,
            options(nostack)
        );
        if status == 0 {
            break;
        }
        let reason = vmx::vmread(VMCS_EXIT_REASON).unwrap_or(0);
        warn!("VM exit reason: {:#x}", reason & 0xffff);
        asm!(
            "vmresume\n\
             setna {status}",
            status = out(reg_byte) status,
            options(nostack)
        );
        if status != 0 {
            break;
        }
    }
}

unsafe fn svm_launch(vmcb: *mut u8) {
    svm::vmrun(vmcb);
    warn!("SVM guest exited");
}

unsafe fn host_state() -> HostState {
    let cr0 = transition::read_cr0();
    let cr3 = transition::read_cr3();
    let cr4 = transition::read_cr4();
    let gdtr = transition::read_gdtr();
    let idtr = transition::read_idtr();
    let (sysenter_cs, sysenter_rsp, sysenter_rip) = transition::read_sysenter();
    HostState {
        cr0,
        cr3,
        cr4,
        rip: vmexit_stub as u64,
        rsp: host_stack_top(),
        gdtr_base: gdtr.base,
        idtr_base: idtr.base,
        sysenter_cs,
        sysenter_rip,
        sysenter_rsp,
    }
}

unsafe fn guest_state() -> GuestState {
    let cr0 = transition::read_cr0();
    let cr3 = transition::read_cr3();
    let cr4 = transition::read_cr4();
    let gdtr = transition::read_gdtr();
    let idtr = transition::read_idtr();
    GuestState {
        cr0,
        cr3,
        cr4,
        rip: guest_entry_stub as u64,
        rsp: guest_stack_top(),
        gdtr_base: gdtr.base,
        idtr_base: idtr.base,
    }
}

unsafe fn host_stack_top() -> u64 {
    HOST_STACK.as_ptr().add(HOST_STACK.len()) as u64
}

unsafe fn guest_stack_top() -> u64 {
    GUEST_STACK.as_ptr().add(GUEST_STACK.len()) as u64
}
