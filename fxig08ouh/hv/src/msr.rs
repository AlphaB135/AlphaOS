//! Minimal helpers for reading/writing MSRs and control registers.

use core::arch::asm;

pub unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}

pub unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nostack)
    );
}

pub unsafe fn read_cr4() -> u64 {
    let value: u64;
    asm!("mov {0}, cr4", out(reg) value, options(nomem, preserves_flags));
    value
}

pub unsafe fn write_cr4(value: u64) {
    asm!("mov cr4, {0}", in(reg) value, options(nostack, preserves_flags));
}
