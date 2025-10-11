#![no_std]
#![no_main]

use core::panic::PanicInfo;

use mk::ipc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {
        if let Some(msg) = ipc::recv() {
            handle_message(msg);
        }
    }
}

fn handle_message(_msg: ipc::Message) {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
