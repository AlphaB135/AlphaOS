#![no_std]

pub mod framebuffer;
pub mod nvme;
pub mod ps2;
pub mod usb;
pub mod virtio_net;

/// Initialize all core drivers needed during early boot.
pub fn init_all() {
    ps2::init();
    nvme::init();
    virtio_net::init();
}
