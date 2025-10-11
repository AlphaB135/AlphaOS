#![no_std]
#![no_main]

use core::panic::PanicInfo;

use posix::abi::{log_write, sleep_ms};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    log_write("Hello from user task\n");
    sleep_ms(1000);
    log_write("Goodbye from user task\n");
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
