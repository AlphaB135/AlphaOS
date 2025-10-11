//! Architecture-specific setup for x86_64 CPUs: GDT/IDT, interrupt enabling, and CPU idling.

pub mod gdt_idt;
pub mod interrupts;
pub mod transition;

pub use transition::{cli, halt, pause, sti};

/// Called once microkernel subsystems are initialized to inform observers (serial/framebuffer).
pub fn announce_ready() {
    // TODO: bridge to framebuffer/serial logging facilities.
}

/// Hint the CPU to sleep until the next interrupt.
pub fn idle() {
    halt();
}
