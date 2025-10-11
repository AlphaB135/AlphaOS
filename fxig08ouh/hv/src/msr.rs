//! Minimal helpers for reading/writing MSRs and control registers.

use core::arch::asm;

pub unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u64;
    let high: u64;
    asm!(
        "rdmsr",
        in("rcx") msr,
        lateout("rax") low,
        lateout("rdx") high,
        options(nostack, preserves_flags)
    );
    (high << 32) | (low & 0xFFFF_FFFF)
}

pub unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value & 0xFFFF_FFFF;
    let high = value >> 32;
    asm!(
        "wrmsr",
        in("rcx") msr,
        in("rax") low,
        in("rdx") high,
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
