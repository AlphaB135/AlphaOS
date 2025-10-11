//! Low-level CPU transition helpers implemented with inline assembly.

use core::arch::asm;

pub const IA32_EFER: u32 = 0xC000_0080;
pub const IA32_LSTAR: u32 = 0xC000_0082;
pub const IA32_FMASK: u32 = 0xC000_0084;
pub const IA32_STAR: u32 = 0xC000_0081;
pub const IA32_SYSENTER_CS: u32 = 0x0000_0174;
pub const IA32_SYSENTER_ESP: u32 = 0x0000_0175;
pub const IA32_SYSENTER_EIP: u32 = 0x0000_0176;
pub const IA32_FS_BASE: u32 = 0xC000_0100;
pub const IA32_GS_BASE: u32 = 0xC000_0101;
pub const IA32_KERNEL_GS_BASE: u32 = 0xC000_0102;

pub const IA32_SYSENTER_CS: u32 = 0x0000_0174;
pub const IA32_SYSENTER_ESP: u32 = 0x0000_0175;
pub const IA32_SYSENTER_EIP: u32 = 0x0000_0176;

const CR0_PG: u64 = 1 << 31;
const CR0_PE: u64 = 1 << 0;
const CR0_WP: u64 = 1 << 16;
const CR4_PAE: u64 = 1 << 5;
const CR4_PGE: u64 = 1 << 7;
const EFER_LME: u64 = 1 << 8;
const EFER_NXE: u64 = 1 << 11;

#[repr(C, packed)]
pub struct DescriptorTable {
    pub limit: u16,
    pub base: u64,
}

/// Read contents of GDTR.
pub unsafe fn read_gdtr() -> DescriptorTable {
    let mut desc = DescriptorTable { limit: 0, base: 0 };
    asm!("sgdt [{0}]", in(reg) &mut desc, options(nostack));
    desc
}

/// Read contents of IDTR.
pub unsafe fn read_idtr() -> DescriptorTable {
    let mut desc = DescriptorTable { limit: 0, base: 0 };
    asm!("sidt [{0}]", in(reg) &mut desc, options(nostack));
    desc
}

/// Read CR0.
pub unsafe fn read_cr0() -> u64 {
    let value: u64;
    asm!("mov {0}, cr0", out(reg) value, options(nomem, preserves_flags));
    value
}

/// Write CR0.
pub unsafe fn write_cr0(value: u64) {
    asm!("mov cr0, {0}", in(reg) value, options(nostack, preserves_flags));
}

/// Read CR3.
pub unsafe fn read_cr3() -> u64 {
    let value: u64;
    asm!("mov {0}, cr3", out(reg) value, options(nomem, preserves_flags));
    value
}

/// Write CR3 with the provided physical address of the root page table.
pub unsafe fn write_cr3(phys: u64) {
    asm!("mov cr3, {0}", in(reg) phys, options(nostack, preserves_flags));
}

/// Read CR4.
pub unsafe fn read_cr4() -> u64 {
    let value: u64;
    asm!("mov {0}, cr4", out(reg) value, options(nomem, preserves_flags));
    value
}

/// Write CR4.
pub unsafe fn write_cr4(value: u64) {
    asm!("mov cr4, {0}", in(reg) value, options(nostack, preserves_flags));
}

/// Flush the entire TLB.
pub unsafe fn flush_tlb() {
    let cr3 = read_cr3();
    write_cr3(cr3);
}

/// Invalidate a single page from the TLB.
pub unsafe fn invlpg(addr: u64) {
    asm!("invlpg [{0}]", in(reg) addr, options(nostack));
}

/// Read an MSR.
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

/// Read SYSENTER configuration registers.
pub unsafe fn read_sysenter() -> (u64, u64, u64) {
    let cs = rdmsr(IA32_SYSENTER_CS);
    let esp = rdmsr(IA32_SYSENTER_ESP);
    let eip = rdmsr(IA32_SYSENTER_EIP);
    (cs, esp, eip)
}

/// Write an MSR.
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

/// Enable paging and long mode once page tables and CR registers are prepared.
pub unsafe fn enable_long_mode(pml4_phys: u64) {
    let mut cr4 = read_cr4();
    cr4 |= CR4_PAE | CR4_PGE;
    write_cr4(cr4);

    let mut efer = rdmsr(IA32_EFER);
    efer |= EFER_LME | EFER_NXE;
    wrmsr(IA32_EFER, efer);

    write_cr3(pml4_phys);

    let mut cr0 = read_cr0();
    cr0 |= CR0_PE | CR0_PG | CR0_WP;
    write_cr0(cr0);
}

/// Enter a long-mode entry point once paging is enabled.
pub unsafe fn jump_long_mode(entry: u64, stack: u64) -> ! {
    let selector = 0x08u16;
    asm!(
        "mov rsp, {stack}\n\
         push {sel}\n\
         push {target}\n\
         retf",
        stack = in(reg) stack,
        sel = in(reg) selector as u64,
        target = in(reg) entry,
        options(noreturn)
    );
}

/// Issue the PAUSE instruction for spin loops.
pub fn pause() {
    unsafe { asm!("pause", options(nomem, preserves_flags)); }
}

/// Halt the CPU until the next interrupt.
pub fn halt() {
    unsafe { asm!("hlt", options(nomem, preserves_flags)); }
}

/// Globally enable interrupts.
pub unsafe fn sti() {
    asm!("sti", options(nomem, preserves_flags));
}

/// Globally disable interrupts.
pub unsafe fn cli() {
    asm!("cli", options(nomem, preserves_flags));
}
