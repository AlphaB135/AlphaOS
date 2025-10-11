#![no_std]

pub mod overlay;
pub mod virtiofs;
pub mod vmm;

/// Initialize the VM subsystem.
pub fn init() {
    hv::init();
    vmm::bootstrap();
}
