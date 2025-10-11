#![no_std]

pub mod attest;
pub mod iommu;
pub mod svm;
pub mod vmx;

use log::info;

/// Initialize virtualization extensions based on CPUID vendor string.
pub fn init() {
    if vmx::is_supported() {
        info!("Enabling Intel VT-x support");
        vmx::init();
    } else if svm::is_supported() {
        info!("Enabling AMD SVM support");
        svm::init();
    } else {
        info!("No hardware virtualization extensions found");
    }
}
