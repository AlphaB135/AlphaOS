#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use gfx::compositor::{self, CursorShape, DisplayCommand};
use mk::ipc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    compositor::init();
    compositor::draw_banner_window();

    loop {
        if let Some(msg) = ipc::recv() {
            handle_message(msg);
        }
    }
}

fn handle_message(msg: ipc::Message) {
    match msg.ty {
        ipc::MessageType::Send => {
            if let Some(cmd) = DisplayCommand::from_payload(msg.payload) {
                compositor::dispatch(cmd);
            }
        }
        ipc::MessageType::Call => {
            // TODO: send reply via IPC once call semantics are defined.
        }
        ipc::MessageType::Recv => {}
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
