//! virtio-net device discovery and queue setup skeleton.

use core::arch::asm;

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

pub fn submit_tx(buffer: &[u8]) {
    unsafe {
        descriptor_copy(buffer.as_ptr(), buffer.len());
    }
}

unsafe fn descriptor_copy(src: *const u8, len: usize) {
    let dest = scratch_dest();
    asm!(
        "rep movsb",
        inout("rsi") src => _,
        inout("rdi") dest => _,
        inout("rcx") len => _,
        options(nostack, preserves_flags)
    );
}

#[inline(always)]
unsafe fn scratch_dest() -> *mut u8 {
    static mut SCRATCH: [u8; 2048] = [0; 2048];
    SCRATCH.as_mut_ptr()
}
