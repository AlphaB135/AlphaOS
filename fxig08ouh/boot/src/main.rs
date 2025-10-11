#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

mod framebuffer;
mod measured_boot;
mod mmap;
mod uefi_entry;

/// Panic handler prints the panic message using the early framebuffer and halts.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    framebuffer::panic_flush(info);
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: uefi::Handle,
    mut system_table: uefi::table::SystemTable<uefi::table::Boot>,
) -> uefi::Status {
    uefi_entry::efi_main(image_handle, &mut system_table)
}
