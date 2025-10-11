//! Window bridge between guest surfaces and compositor.

pub fn init() {
    // TODO: register overlay surfaces with gfx compositor once VM boots.
}

pub fn map_surface(_guest_phys: u64) {
    // TODO: translate guest memory into host framebuffer overlay.
}
