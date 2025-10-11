//! Interrupt controller bring-up for APIC + HPET wiring.

use core::arch::asm;

/// Enable interrupts globally.
pub unsafe fn enable() {
    asm!("sti", options(nomem, preserves_flags));
}

/// Disable interrupts globally.
pub unsafe fn disable() {
    asm!("cli", options(nomem, preserves_flags));
}

/// Save general-purpose registers on interrupt entry.
pub unsafe fn isr_prologue() {
    asm!(
        "push rax\n\
         push rcx\n\
         push rdx\n\
         push rbx\n\
         push rbp\n\
         push rsi\n\
         push rdi\n\
         push r8\n\
         push r9\n\
         push r10\n\
         push r11",
        options(nostack)
    );
}

/// Restore general-purpose registers before `iretq`.
pub unsafe fn isr_epilogue() {
    asm!(
        "pop r11\n\
         pop r10\n\
         pop r9\n\
         pop r8\n\
         pop rdi\n\
         pop rsi\n\
         pop rbp\n\
         pop rbx\n\
         pop rdx\n\
         pop rcx\n\
         pop rax",
        options(nostack)
    );
}
