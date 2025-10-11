//! Attestation hub bridging hypervisor state to remote verifier.

/// Record measurement data for guest launch.
pub fn record_launch_digest() {
    // TODO: integrate with TPM quotes.
}

/// Provide attestation report for the guest agent.
pub fn report() {
    // TODO: serialize measurements over micro-IPC channel.
}
