//! Capability manifest model per task.

use heapless::Vec;
use mk::caps::CapabilityClass;

const MAX_ENTRIES: usize = 8;

#[derive(Clone, Debug)]
pub struct ManifestEntry {
    pub task: u32,
    pub capabilities: CapabilityClass,
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry, MAX_ENTRIES>,
}

impl Manifest {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, entry: ManifestEntry) -> bool {
        self.entries.push(entry).is_ok()
    }

    pub fn allowed(&self, task: u32, required: CapabilityClass) -> bool {
        self.entries
            .iter()
            .find(|entry| entry.task == task)
            .map(|entry| entry.capabilities.contains(required))
            .unwrap_or(false)
    }
}
