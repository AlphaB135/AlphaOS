//! GDT and IDT bootstrap stubs. Replace with concrete descriptor tables during bring-up.

use spin::Once;
use x86_64::instructions::segmentation::{CS, Segment, DS, ES, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use super::interrupts;
use crate::time::apic;

static GDT: Once<(GlobalDescriptorTable, SegmentSelector, SegmentSelector)> = Once::new();
static IDT: Once<InterruptDescriptorTable> = Once::new();

/// Install provisional descriptor tables.
pub unsafe fn install() {
    let gdt = GDT.call_once(|| {
        let mut table = GlobalDescriptorTable::new();
        let code = table.append(Descriptor::kernel_code_segment());
        let data = table.append(Descriptor::kernel_data_segment());
        (table, code, data)
    });

    let table = &gdt.0;
    let code_selector = gdt.1;
    let data_selector = gdt.2;

    unsafe {
        table.load();
        CS::set_reg(code_selector);
        DS::set_reg(data_selector);
        ES::set_reg(data_selector);
        SS::set_reg(data_selector);
    }

    let idt = IDT.call_once(|| {
        let mut table = InterruptDescriptorTable::new();
        table.divide_error.set_handler_fn(divide_by_zero);
        table
    });

    unsafe { idt.load(); }
}

extern "x86-interrupt" fn divide_by_zero(stack: InterruptStackFrame) {
    unsafe {
        interrupts::isr_prologue();
    }
    log::error!("Divide-by-zero at {:#x}", stack.instruction_pointer.as_u64());
    apic::eoi();
    unsafe {
        interrupts::isr_epilogue();
    }
}
