//! virtio-fs planning stubs.

/// Placeholder for virtio-fs daemon wiring.
pub fn init() {
    // TODO: map shared memory window between host and guest.
}

/// Future design notes:
/// - Use DAX window for zero-copy framebuffer overlay into guest.
/// - Provide capability for guest agent to request GPU surfaces.
