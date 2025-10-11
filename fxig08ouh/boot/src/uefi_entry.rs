use mk::{BootInfo, FrameBufferInfo};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::Status;

use security::install_default_manifest;

use crate::framebuffer::{self, BootFrameBuffer};
use crate::measured_boot;
use crate::mmap;

/// Primary firmware entrypoint. Collects boot services information and transfers control to the microkernel.
pub fn efi_main(image_handle: Handle, st: &mut SystemTable<Boot>) -> Status {
    let _ = image_handle;

    framebuffer::init_serial(0x3F8);

    install_default_manifest();

    if let Ok(gop) = st.boot_services().locate_protocol::<GraphicsOutput>() {
        // SAFETY: firmware guarantees the protocol pointer remains valid while BootServices are alive.
        let gop = unsafe { &mut *gop.get() };
        let fb = BootFrameBuffer::new(gop.frame_buffer(), gop);
        framebuffer::install(fb);
        framebuffer::draw_banner();
    }

    let memory_regions = mmap::collect_memory_map(st.boot_services());

    let boot_info = BootInfo {
        memory_regions,
        framebuffer: framebuffer::current().map(|fb| FrameBufferInfo { ..fb.info }),
        rsdp_addr: locate_rsdp(st).ok(),
    };

    measured_boot::measure_boot_info(st, &boot_info);

    mk::init(boot_info)
}

fn locate_rsdp(_st: &SystemTable<Boot>) -> Result<u64, Status> {
    // TODO: Parse ACPI tables via `st.config_table()` when running on hardware.
    Err(Status::NOT_FOUND)
}
