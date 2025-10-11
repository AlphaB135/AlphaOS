//! Lightweight task registry tracking metadata and capability profiles.

use heapless::Vec;
use spin::Mutex;

use crate::caps::{self, Capability, CapabilityClass};
use crate::sched::TaskId;

const MAX_TASKS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Registered,
    Active,
    Suspended,
}

#[derive(Clone, Copy, Debug)]
pub struct TaskProfile {
    pub id: TaskId,
    pub name: [u8; 16],
    pub allowed: CapabilityClass,
    pub state: TaskState,
}

static TASKS: Mutex<Vec<TaskProfile, MAX_TASKS>> = Mutex::new(Vec::new());

pub fn register(id: TaskId, name: &[u8], allowed: CapabilityClass) -> bool {
    let mut entry = [0u8; 16];
    let copy_len = core::cmp::min(name.len(), entry.len());
    entry[..copy_len].copy_from_slice(&name[..copy_len]);
    let profile = TaskProfile {
        id,
        name: entry,
        allowed,
        state: TaskState::Registered,
    };
    let mut tasks = TASKS.lock();
    if tasks.iter().any(|task| task.id == id) {
        return false;
    }
    let inserted = tasks.push(profile).is_ok();
    if inserted {
        grant_allowed_caps(id, allowed);
    }
    inserted
}

pub fn activate(id: TaskId) -> bool {
    let mut tasks = TASKS.lock();
    if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
        task.state = TaskState::Active;
        true
    } else {
        false
    }
}

pub fn suspend(id: TaskId) -> bool {
    let mut tasks = TASKS.lock();
    if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
        task.state = TaskState::Suspended;
        task.allowed = CapabilityClass::empty();
        caps::revoke(id, None);
        true
    } else {
        false
    }
}

pub fn profile(id: TaskId) -> Option<TaskProfile> {
    TASKS.lock().iter().cloned().find(|task| task.id == id)
}

pub fn grant_allowed_caps(id: TaskId, allowed: CapabilityClass) {
    caps::revoke(id, None);
    const CLASSES: [CapabilityClass; 7] = [
        CapabilityClass::FILE,
        CapabilityClass::NET,
        CapabilityClass::GPU,
        CapabilityClass::INPUT,
        CapabilityClass::TIME,
        CapabilityClass::IRQ,
        CapabilityClass::MM,
    ];
    for class in CLASSES.iter() {
        if allowed.contains(*class) {
            let cap = Capability {
                owner: id,
                class: *class,
                object: 0,
            };
            let _ = caps::grant(cap);
        }
    }
}

pub fn revoke_all(id: TaskId) {
    // TODO: maintain per-task capability list to revoke precisely.
    let mut tasks = TASKS.lock();
    if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
        task.allowed = CapabilityClass::empty();
        task.state = TaskState::Suspended;
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::caps;

    fn allow_all(_: TaskId, _: Capability) -> bool {
        true
    }

    #[test]
    fn register_and_lookup_profile() {
        caps::init();
        caps::register_policy(allow_all);
        assert!(register(TaskId(10), b"display", CapabilityClass::GPU | CapabilityClass::INPUT));
        activate(TaskId(10));
        let profile = profile(TaskId(10)).expect("profile");
        assert_eq!(profile.state, TaskState::Active);
        assert!(profile.allowed.contains(CapabilityClass::GPU));
    }

    #[test]
    fn suspend_revokes_capabilities() {
        caps::init();
        caps::register_policy(allow_all);
        assert!(register(TaskId(20), b"net", CapabilityClass::NET | CapabilityClass::MM));
        activate(TaskId(20));
        assert!(caps::has(TaskId(20), CapabilityClass::NET));
        suspend(TaskId(20));
        assert!(!caps::has(TaskId(20), CapabilityClass::NET));
        assert_eq!(profile(TaskId(20)).unwrap().state, TaskState::Suspended);
    }
}
