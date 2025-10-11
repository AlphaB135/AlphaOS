#![no_std]

pub mod attest;
pub mod iommu;
pub mod msr;
pub mod svm;
pub mod vmx;

use log::info;

/// Initialize virtualization extensions based on CPUID vendor string.
pub fn init() {
    if vmx::is_supported() {
        info!("Enabling Intel VT-x support");
        if let Err(e) = vmx::init() {
            info!("VMX init failed: {:?}", e);
        }
    } else if svm::is_supported() {
        info!("Enabling AMD SVM support");
        if let Err(e) = svm::init() {
            info!("SVM init failed: {:?}", e);
        }
    } else {
        info!("No hardware virtualization extensions found");
    }
}
