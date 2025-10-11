#![no_std]
#![no_main]

use core::panic::PanicInfo;

use drivers::ps2;
use mk::ipc::{self, Message, MessageType};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    ps2::init();

    loop {
        if let Some(code) = ps2::poll_scancode() {
            dispatch_key(code);
        }
    }
}

fn dispatch_key(scancode: u8) {
    let message = Message {
        ty: MessageType::Send,
        src: 2,
        dst: 1,
        payload: [scancode as u64, 0, 0, 0],
    };
    let _ = ipc::send(message);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
