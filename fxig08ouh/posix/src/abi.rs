//! POSIX-like syscall numbers and helper wrappers.

pub const SYS_IPC_SEND: u16 = 0;
pub const SYS_IPC_RECV: u16 = 1;
pub const SYS_CAP_GRANT: u16 = 2;
pub const SYS_SLEEP_MS: u16 = 3;
pub const SYS_LOG_WRITE: u16 = 4;

pub fn sleep_ms(ms: u64) {
    unsafe {
        syscall(SYS_SLEEP_MS, ms, 0, 0, 0);
    }
}

pub fn log_write(message: &str) {
    for chunk in message.as_bytes().chunks(8) {
        let mut buf = [0u8; 8];
        buf[..chunk.len()].copy_from_slice(chunk);
        let value = u64::from_le_bytes(buf);
        unsafe {
            syscall(SYS_LOG_WRITE, value, chunk.len() as u64, 0, 0);
        }
    }
}

/// SAFETY: must be called according to architecture ABI once syscall instruction exists.
unsafe fn syscall(number: u16, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let _ = (number, arg0, arg1, arg2, arg3);
    // TODO: issue `syscall` instruction once IDT vectors mapped.
    0
}
