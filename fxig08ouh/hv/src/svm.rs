//! AMD SVM initialization stubs.

/// Check CPUID vendor string for "AuthenticAMD" and SVM bit.
pub fn is_supported() -> bool {
    false
}

/// Initialize VMCB structures.
pub fn init() {
    // TODO: allocate VMCB and set intercept vectors.
}
