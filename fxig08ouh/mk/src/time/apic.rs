//! Local APIC timer configuration and calibration.

use crate::arch::transition::{rdmsr, wrmsr};

const IA32_APIC_BASE: u32 = 0x1B;
const APIC_ENABLE: u64 = 1 << 11;
const LAPIC_EOI_OFFSET: usize = 0x0B0;
const LAPIC_TIMER_DIV: usize = 0x3E0;
const LAPIC_TIMER_INIT: usize = 0x380;
const LAPIC_TIMER_CURR: usize = 0x390;

static mut LAPIC_BASE: usize = 0;

/// Initialize the local APIC timer in periodic mode.
pub fn init() {
    unsafe {
        enable_xapic();
        program_timer();
    }
}

/// Issue End-Of-Interrupt to the local APIC.
pub fn eoi() {
    unsafe {
        if LAPIC_BASE != 0 {
            core::ptr::write_volatile((LAPIC_BASE + LAPIC_EOI_OFFSET) as *mut u32, 0);
        }
    }
}

unsafe fn enable_xapic() {
    let base = rdmsr(IA32_APIC_BASE) | APIC_ENABLE;
    wrmsr(IA32_APIC_BASE, base);
    LAPIC_BASE = (base as usize) & 0xFFFF_F000;
}

unsafe fn program_timer() {
    if LAPIC_BASE == 0 {
        return;
    }
    let lapic = LAPIC_BASE as *mut u32;
    core::ptr::write_volatile(lapic.add(LAPIC_TIMER_DIV / 4), 0b1011);
    core::ptr::write_volatile(lapic.add(LAPIC_TIMER_INIT / 4), 0x00FF_FFFF);
    // Let the timer run briefly and sample the current count to derive calibration later.
    let _current = core::ptr::read_volatile(lapic.add(LAPIC_TIMER_CURR / 4));
}
