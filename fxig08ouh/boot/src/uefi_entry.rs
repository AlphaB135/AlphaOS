use alloc::vec::Vec;
use core::mem::size_of;

use mk::{BootInfo, FrameBufferInfo};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{MemoryMapSize, MemoryType};
use uefi::Status;

use security::install_default_manifest;

use crate::framebuffer::{self, BootFrameBuffer};
use crate::loader;
use crate::measured_boot;

/// Primary firmware entrypoint. Collects boot services information and transfers control to the microkernel.
pub fn efi_main(image_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    framebuffer::init_serial(0x3F8);
    install_default_manifest();

    // Locate the framebuffer before exiting boot services.
    let framebuffer_info = if let Ok(gop) = st.boot_services().locate_protocol::<GraphicsOutput>() {
        let gop = unsafe { &mut *gop.get() };
        let fb = BootFrameBuffer::new(gop.frame_buffer(), gop);
        framebuffer::install(fb);
        framebuffer::draw_banner();
        framebuffer::current().map(|fb| FrameBufferInfo { ..fb.info })
    } else {
        None
    };

    let rsdp_addr = locate_rsdp(&st).ok();

    // Grab a preliminary memory map so we can perform TPM measurements before exiting boot services.
    if let Ok(boot_info_for_measure) = preliminary_boot_info(&mut st, framebuffer_info, rsdp_addr) {
        measured_boot::measure_boot_info(&mut st, &boot_info_for_measure);
    }

    // Exit boot services and obtain the final memory map in one step.
    let (mut _runtime_st, memory_map) = st.exit_boot_services(MemoryType::LOADER_DATA);
    let boot_info = loader::build_boot_info(&memory_map, framebuffer_info, rsdp_addr);

    mk::init(boot_info)
}

fn preliminary_boot_info(
    st: &mut SystemTable<Boot>,
    framebuffer: Option<FrameBufferInfo>,
    rsdp: Option<u64>,
) -> Result<BootInfo, Status> {
    let MemoryMapSize { map_size, entry_size } = st.boot_services().memory_map_size();
    let buffer_len = map_size + entry_size * 8; // extra slack for map growth
    let usize_count = (buffer_len + size_of::<usize>() - 1) / size_of::<usize>();
    let mut buffer: Vec<usize> = Vec::with_capacity(usize_count);
    unsafe { buffer.set_len(usize_count); }
    let raw_buf = unsafe {
        core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer_len)
    };

    let memory_map = st.boot_services().memory_map(raw_buf)?;
    Ok(loader::build_boot_info(&memory_map, framebuffer, rsdp))
}

fn locate_rsdp(_st: &SystemTable<Boot>) -> Result<u64, Status> {
    // TODO: Parse ACPI tables via `st.config_table()` when running on hardware.
    Err(Status::NOT_FOUND)
}
