//! Hooks security manifests into the microkernel capability table and task registry.

use mk::caps::{self, Capability, CapabilityClass};
use mk::sched::TaskId;
use mk::task;
use spin::Once;

use crate::sandbox::{Manifest, ManifestEntry};

static MANIFEST: Once<Manifest> = Once::new();

pub fn install_default_manifest() {
    let manifest = MANIFEST.call_once(default_manifest);
    register_tasks(manifest);
    grant_manifest_caps(manifest);
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

fn register_tasks(manifest: &Manifest) {
    for entry in manifest.entries.iter() {
        let name = match entry.task {
            1 => b"display",
            2 => b"input",
            3 => b"fs",
            _ => b"task",
        };
        let _ = task::register(TaskId(entry.task), name, entry.capabilities);
    }
}

fn grant_manifest_caps(manifest: &Manifest) {
    const CLASSES: [CapabilityClass; 7] = [
        CapabilityClass::FILE,
        CapabilityClass::NET,
        CapabilityClass::GPU,
        CapabilityClass::INPUT,
        CapabilityClass::TIME,
        CapabilityClass::IRQ,
        CapabilityClass::MM,
    ];
    for entry in manifest.entries.iter() {
        for class in CLASSES.iter() {
            if entry.capabilities.contains(*class) {
                let capability = Capability {
                    owner: TaskId(entry.task),
                    class: *class,
                    object: 0,
                };
                let _ = caps::grant(capability);
            }
        }
    }
}
