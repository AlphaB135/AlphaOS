#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use drivers::nvme;
use heapless::{String, Vec};
use mk::ipc;
use spin::Mutex;

const MAX_KV: usize = 16;
const VALUE_BYTES: usize = 256;

static KV_STORE: Mutex<Vec<(String<32>, [u8; VALUE_BYTES]), MAX_KV>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn _start() -> ! {
    nvme::init();
    if let Ok(identify) = nvme::identify_controller() {
        nvme::log_identify(&identify);
    }

    loop {
        if let Some(msg) = ipc::recv() {
            handle_message(msg);
        }
    }
}

fn handle_message(msg: ipc::Message) {
    match msg.payload[0] {
        0 => {
            let lba = msg.payload[1];
            let mut buffer = [0u8; VALUE_BYTES];
            if nvme::read_block(lba, &mut buffer).is_ok() {
                nvme::log_sector(&buffer);
            }
        }
        1 => {
            let key_char = (msg.payload[1] as u8) as char;
            let mut key = String::<32>::new();
            let _ = key.push(key_char);
            let mut store = KV_STORE.lock();
            let _ = store.push((key, [0u8; VALUE_BYTES]));
        }
        _ => {}
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
