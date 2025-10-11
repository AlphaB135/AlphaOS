//! Syscall dispatch table and ABI definitions for the POSIX shim.

use crate::caps::{self, Capability, CapabilityClass};
use crate::ipc::{recv, send, Message, MessageType};
use crate::sched::TaskId;

#[repr(u16)]
pub enum SyscallNumber {
    IpcSend = 0,
    IpcRecv = 1,
    CapGrant = 2,
    SleepMs = 3,
    LogWrite = 4,
}

pub fn install() {
    // TODO: hook up IDT entries for syscall/sysret or sysenter.
}

pub fn dispatch(num: SyscallNumber, args: [u64; 4]) -> u64 {
    match num {
        SyscallNumber::IpcSend => {
            let msg = Message {
                ty: MessageType::Send,
                src: args[0] as u32,
                dst: args[1] as u32,
                payload: [args[2], args[3], 0, 0],
            };
            send(msg) as u64
        }
        SyscallNumber::IpcRecv => recv().map(|m| m.src as u64).unwrap_or(u64::MAX),
        SyscallNumber::CapGrant => grant_cap(TaskId(args[0] as u32), args[1] as u32, args[2]) as u64,
        SyscallNumber::SleepMs => {
            crate::time::hpet::sleep_ms(args[0]);
            0
        }
        SyscallNumber::LogWrite => 0,
    }
}

fn grant_cap(task: TaskId, class_bits: u32, object: u64) -> bool {
    let Some(class) = CapabilityClass::from_bits(class_bits) else {
        return false;
    };
    let capability = Capability {
        owner: task,
        class,
        object,
    };
    caps::grant(capability)
}
