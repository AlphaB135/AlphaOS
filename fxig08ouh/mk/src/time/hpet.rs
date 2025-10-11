//! HPET initialization skeleton. Programs the main counter for millisecond resolution.

use core::arch::asm;

static mut HPET_BASE: usize = 0;

/// Configure HPET registers.
pub fn init() {
    unsafe {
        memory_barrier();
        // TODO: map HPET via physical addresses from ACPI tables and program comparator.
    }
}

/// Busy wait using HPET for `millis`.
pub fn sleep_ms(millis: u64) {
    unsafe {
        memory_barrier();
        if HPET_BASE == 0 {
            return;
        }
        let start = read_main_counter();
        let target = start.wrapping_add(millis * ticks_per_ms());
        while read_main_counter().wrapping_sub(target) as i64 <= 0 {
            asm!("pause", options(nomem, preserves_flags));
        }
    }
}

unsafe fn read_main_counter() -> u64 {
    core::ptr::read_volatile((HPET_BASE + 0xF0) as *const u64)
}

const fn ticks_per_ms() -> u64 {
    // Placeholder 1 MHz clock until calibrated.
    1_000_000
}

#[inline(always)]
unsafe fn memory_barrier() {
    asm!("lfence", options(nomem, preserves_flags));
}
