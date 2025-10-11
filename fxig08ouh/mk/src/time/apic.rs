//! Local APIC timer configuration and calibration.

/// Initialize the local APIC timer in periodic mode.
pub fn init() {
    // TODO: Program LAPIC registers once MMIO mapping is in place.
}

/// Issue End-Of-Interrupt to the local APIC.
pub fn eoi() {
    // TODO: Write to LAPIC EOI register.
}
