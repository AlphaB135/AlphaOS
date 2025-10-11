//! GDT and IDT bootstrap stubs. Replace with concrete descriptor tables during bring-up.

use spin::Once;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::idt::InterruptDescriptorTable;

static GDT: Once<GlobalDescriptorTable> = Once::new();
static IDT: Once<InterruptDescriptorTable> = Once::new();

/// Install provisional descriptor tables.
pub unsafe fn install() {
    let gdt = GDT.call_once(GlobalDescriptorTable::new);
    gdt.load();

    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        // TODO: populate fault handlers.
        idt
    });
    idt.load();
}
