//! GDT and IDT bootstrap stubs. Replace with concrete descriptor tables during bring-up.

use core::arch::asm;

use spin::Once;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::DescriptorTablePointer;

use super::interrupts;
use crate::time::apic;

static GDT: Once<GlobalDescriptorTable> = Once::new();
static IDT: Once<InterruptDescriptorTable> = Once::new();

/// Install provisional descriptor tables.
pub unsafe fn install() {
    let gdt = GDT.call_once(|| {
        let mut table = GlobalDescriptorTable::new();
        table.add_entry(Descriptor::kernel_code_segment());
        table.add_entry(Descriptor::kernel_data_segment());
        table
    });

    let gdt_ptr = gdt.pointer();
    load_gdt(&gdt_ptr);

    let idt = IDT.call_once(|| {
        let mut table = InterruptDescriptorTable::new();
        table.divide_error.set_handler_fn(divide_by_zero);
        table
    });

    let idt_ptr = idt.pointer();
    load_idt(&idt_ptr);
}

/// SAFETY: caller ensures pointer references a valid GDT descriptor.
unsafe fn load_gdt(ptr: &DescriptorTablePointer) {
    asm!("lgdt [{0}]", in(reg) ptr, options(readonly, nostack));
}

/// SAFETY: caller ensures pointer references a valid IDT descriptor.
unsafe fn load_idt(ptr: &DescriptorTablePointer) {
    asm!("lidt [{0}]", in(reg) ptr, options(readonly, nostack));
}

extern "x86-interrupt" fn divide_by_zero(stack: &mut InterruptStackFrame) {
    unsafe {
        interrupts::isr_prologue();
    }
    log::error!("Divide-by-zero at {:#x}", stack.instruction_pointer.as_u64());
    apic::eoi();
    unsafe {
        interrupts::isr_epilogue();
    }
}
