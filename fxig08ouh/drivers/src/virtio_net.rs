//! virtio-net device discovery and queue setup skeleton.

#[derive(Debug)]
pub struct Features {
    pub negotiate: u64,
}

static mut FEATURES: Features = Features { negotiate: 0 };

pub fn init() {
    unsafe {
        FEATURES.negotiate = 0x1 | 0x20; // Placeholder feature bits.
    }
}

pub fn log_features() {
    unsafe {
        log::info!("virtio-net features negotiated: {:#x}", FEATURES.negotiate);
    }
}

pub fn submit_tx(_buffer: &[u8]) {
    // TODO: place descriptor into TX ring.
}
