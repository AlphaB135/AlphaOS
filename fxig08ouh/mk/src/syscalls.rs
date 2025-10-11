//! Syscall dispatch table and ABI definitions for the POSIX shim.

use core::arch::asm;

use crate::arch::transition::{self, IA32_FMASK, IA32_LSTAR, IA32_STAR};
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
    unsafe {
        // Program STAR/LSTAR with a dummy handler until real syscall entry is hooked.
        let kernel_cs = 0x08u64 << 32;
        let user_cs = (0x1Bu64) << 48;
        transition::wrmsr(IA32_STAR, kernel_cs | user_cs);
        transition::wrmsr(IA32_LSTAR, syscall_entry as u64);
        transition::wrmsr(IA32_FMASK, 0);
    }
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

#[no_mangle]
pub extern "C" fn syscall_entry() -> ! {
    unsafe {
        asm!(
            "swapgs\n\
             push r11\n\
             push rcx\n\
             push rdx\n\
             push rsi\n\
             push rdi\n\
             push r8\n\
             push r9\n\
             push r10\n\
             mov rdi, rax\n\
             mov rsi, rcx\n\
             mov rdx, rdx\n\
             mov r10, r10\n\
             mov r8, r8\n\
             mov r9, r9\n\
             // TODO: call into Rust dispatcher once calling convention is finalised\n\
             pop r10\n\
             pop r9\n\
             pop r8\n\
             pop rdi\n\
             pop rsi\n\
             pop rdx\n\
             pop rcx\n\
             pop r11\n\
             swapgs\n\
             sysretq",
            options(noreturn)
        );
    }
}
