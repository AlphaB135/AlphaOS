//! Policy engine stub that validates manifests against static rules.

use mk::caps::CapabilityClass;

use crate::sandbox::ManifestEntry;

pub fn approve(entry: &ManifestEntry) -> bool {
    let allowed = CapabilityClass::FILE
        | CapabilityClass::GPU
        | CapabilityClass::INPUT
        | CapabilityClass::TIME
        | CapabilityClass::IRQ
        | CapabilityClass::MM;
    entry.capabilities.bits() & allowed.bits() == entry.capabilities.bits()
}
