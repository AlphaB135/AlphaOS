#![no_std]
#![cfg_attr(test, allow(clippy::unwrap_used))]

extern crate alloc;

pub mod arch;
pub mod caps;
pub mod ipc;
pub mod mem;
pub mod sched;
pub mod syscalls;
pub mod time;

use heapless::Vec;
use linked_list_allocator::LockedHeap;
use spin::Once;

/// Global heap allocator, initialized once paging is active.
#[global_allocator]
static KERNEL_HEAP: LockedHeap = LockedHeap::empty();

static BOOT_INFO: Once<BootInfo> = Once::new();

pub const MAX_MEMORY_REGIONS: usize = 256;


/// Boot-time framebuffer description provided by the loader.
#[derive(Clone, Copy, Debug)]
pub struct FrameBufferInfo {
    pub width: usize,
    pub height: usize,
    pub pixel_format: PixelFormat,
    pub size: usize,
    pub phys_addr: u64,
}

/// Minimal pixel formats supported during bring-up.
#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    Bitmask,
    BltOnly,
}

/// Firmware-reported memory classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    Mmio,
    FirmwareReserved,
}

/// Descriptor translating firmware memory map into kernel-understandable layout.
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub kind: MemoryRegionKind,
}

pub type BootMemoryMap = Vec<MemoryRegion, MAX_MEMORY_REGIONS>;

/// Aggregated boot payload consumed by the microkernel.
#[derive(Clone, Debug)]
pub struct BootInfo {
    pub memory_regions: BootMemoryMap,
    pub framebuffer: Option<FrameBufferInfo>,
    pub rsdp_addr: Option<u64>,
}

/// Entry point from the UEFI loader.
pub fn init(boot_info: BootInfo) -> ! {
    BOOT_INFO.call_once(|| boot_info);

    unsafe {
        mem::heap::bootstrap_global_heap(&KERNEL_HEAP);
    }

    unsafe {
        arch::gdt_idt::install();
        arch::interrupts::enable();
    }

    mem::init(&get_boot_info());
    time::init();
    sched::init();

    ipc::bootstrap();
    caps::init();

    syscalls::install();

    arch::announce_ready();

    loop {
        sched::tick();
        arch::idle();
    }
}

/// Borrow the boot info captured during initialization.
pub fn get_boot_info() -> &'static BootInfo {
    BOOT_INFO.get().expect("boot info must be installed")
}
