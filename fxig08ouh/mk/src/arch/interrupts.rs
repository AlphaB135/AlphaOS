//! Interrupt controller bring-up for APIC + HPET wiring.

/// Enable interrupts globally.
pub unsafe fn enable() {
    core::arch::asm!("sti", options(nomem, preserves_flags));
}

/// Disable interrupts globally.
pub unsafe fn disable() {
    core::arch::asm!("cli", options(nomem, preserves_flags));
}
