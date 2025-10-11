//! Hooks security manifests into the microkernel capability table.

use mk::caps::{self, Capability, CapabilityClass};
use mk::sched::TaskId;
use spin::Once;

use crate::sandbox::{Manifest, ManifestEntry};

static MANIFEST: Once<Manifest> = Once::new();

pub fn install_default_manifest() {
    let _ = MANIFEST.call_once(default_manifest);
    caps::register_policy(policy_check);
}

fn policy_check(task: TaskId, cap: Capability) -> bool {
    manifest().allowed(task.0, cap.class)
}

fn manifest() -> &'static Manifest {
    MANIFEST
        .get()
        .expect("security manifest must be installed before capability checks")
}

fn default_manifest() -> Manifest {
    let mut manifest = Manifest::new();
    let _ = manifest.add(ManifestEntry {
        task: 1,
        capabilities: CapabilityClass::GPU | CapabilityClass::INPUT | CapabilityClass::TIME,
    });
    let _ = manifest.add(ManifestEntry {
        task: 2,
        capabilities: CapabilityClass::INPUT,
    });
    let _ = manifest.add(ManifestEntry {
        task: 3,
        capabilities: CapabilityClass::FILE | CapabilityClass::MM,
    });
    manifest
}
