//! PS/2 controller polling.

use core::sync::atomic::{AtomicU8, Ordering};

static LAST_SCANCODE: AtomicU8 = AtomicU8::new(0);

pub fn init() {
    // TODO: set controller command byte, enable IRQs.
}

pub fn poll_scancode() -> Option<u8> {
    let scancode = LAST_SCANCODE.load(Ordering::Relaxed);
    if scancode == 0 {
        None
    } else {
        LAST_SCANCODE.store(0, Ordering::Relaxed);
        Some(scancode)
    }
}

pub fn inject_scancode(scancode: u8) {
    LAST_SCANCODE.store(scancode, Ordering::Relaxed);
}
