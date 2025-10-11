//! Capability table skeleton for Week-1/2 MVP.

use bitflags::bitflags;
use heapless::Vec;
use spin::Mutex;

use crate::sched::TaskId;

bitflags! {
    pub struct CapabilityClass: u32 {
        const FILE = 0b0000_0001;
        const NET = 0b0000_0010;
        const GPU = 0b0000_0100;
        const INPUT = 0b0000_1000;
        const TIME = 0b0001_0000;
        const IRQ = 0b0010_0000;
        const MM = 0b0100_0000;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Capability {
    pub owner: TaskId,
    pub class: CapabilityClass,
    pub object: u64,
}

const MAX_CAPS: usize = 32;

static CAP_TABLE: Mutex<Vec<Capability, MAX_CAPS>> = Mutex::new(Vec::new());

type PolicyFn = fn(TaskId, Capability) -> bool;

static POLICY: Mutex<Option<PolicyFn>> = Mutex::new(None);

pub fn init() {
    CAP_TABLE.lock().clear();
}

pub fn register_policy(policy: PolicyFn) {
    *POLICY.lock() = Some(policy);
}

pub fn grant(cap: Capability) -> bool {
    if !policy_allows(cap.owner, cap) {
        return false;
    }
    CAP_TABLE.lock().push(cap).is_ok()
}

pub fn has(task: TaskId, class: CapabilityClass) -> bool {
    CAP_TABLE
        .lock()
        .iter()
        .any(|cap| cap.owner == task && cap.class.contains(class))
}

fn policy_allows(task: TaskId, cap: Capability) -> bool {
    if let Some(policy) = *POLICY.lock() {
        policy(task, cap)
    } else {
        true
    }
}

pub fn revoke(task: TaskId, class: Option<CapabilityClass>) {
    CAP_TABLE.lock().retain(|cap| {
        if cap.owner != task {
            return true;
        }
        if let Some(target) = class {
            !cap.class.contains(target)
        } else {
            false
        }
    });
}
