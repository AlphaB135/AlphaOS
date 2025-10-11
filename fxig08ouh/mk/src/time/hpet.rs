//! HPET initialization skeleton. Programs the main counter for millisecond resolution.

/// Configure HPET registers.
pub fn init() {
    // TODO: Map HPET via physical addresses from ACPI tables and program comparator.
}

/// Busy wait using HPET for `millis`.
pub fn sleep_ms(_millis: u64) {
    // TODO: Use HPET counter to implement precise sleeping.
}
