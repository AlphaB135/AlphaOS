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
            "swapgs",
            "push r11",
            "push rcx",
            "push rbx",
            "push rbp",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov rbx, rdi",
            "mov rbp, rsi",
            "mov r12, rdx",
            "mov r13, r10",
            "mov r14, r8",
            "mov r15, r9",
            "mov rdi, rax",
            "mov rsi, rbx",
            "mov rdx, rbp",
            "mov rcx, r12",
            "mov r8, r13",
            "mov r9, r14",
            "sub rsp, 8",
            "mov [rsp], r15",
            "call {handler}",
            "add rsp, 8",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbp",
            "pop rbx",
            "pop rcx",
            "pop r11",
            "swapgs",
            "sysretq",
            handler = sym handle_syscall,
            options(noreturn)
        );
    }
}

fn handle_syscall(number: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    let Some(num) = SyscallNumber::from_u64(number) else {
        return u64::MAX;
    };
    let _ = (arg4, arg5);
    let args = [arg0, arg1, arg2, arg3];
    dispatch(num, args)
}

impl SyscallNumber {
    fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(SyscallNumber::IpcSend),
            1 => Some(SyscallNumber::IpcRecv),
            2 => Some(SyscallNumber::CapGrant),
            3 => Some(SyscallNumber::SleepMs),
            4 => Some(SyscallNumber::LogWrite),
            _ => None,
        }
    }
}
