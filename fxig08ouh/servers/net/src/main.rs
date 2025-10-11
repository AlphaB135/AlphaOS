#![no_std]
#![no_main]

use core::panic::PanicInfo;

use drivers::virtio_net;
use mk::ipc;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    virtio_net::init();
    virtio_net::log_features();

    loop {
        if let Some(msg) = ipc::recv() {
            handle_message(msg);
        }
    }
}

fn handle_message(_msg: ipc::Message) {
    // TODO: push packets to virtio TX ring.
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
