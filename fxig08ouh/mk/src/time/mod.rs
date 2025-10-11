//! Timer management (HPET + APIC calibration).

pub mod apic;
pub mod hpet;

/// Initialize HPET and APIC timer plumbing.
pub fn init() {
    hpet::init();
    apic::init();
}

/// Called on every timer tick interrupt to advance scheduler clocks.
pub fn tick() {
    // TODO: integrate with scheduler accounting.
}
