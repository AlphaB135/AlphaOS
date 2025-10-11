//! Micro-IPC primitives (Send, Recv, Call) using bounded queues.

use heapless::Deque;
use spin::Mutex;

#[derive(Clone, Copy, Debug)]
pub enum MessageType {
    Send,
    Recv,
    Call,
}

#[derive(Clone, Copy, Debug)]
pub struct Message {
    pub ty: MessageType,
    pub src: u32,
    pub dst: u32,
    pub payload: [u64; 4],
}

const QUEUE_DEPTH: usize = 16;

static QUEUE: Mutex<Deque<Message, QUEUE_DEPTH>> = Mutex::new(Deque::new());

/// Prepare IPC data structures.
pub fn bootstrap() {
    QUEUE.lock().clear();
}

/// Send a message, returning `false` when the queue is full.
pub fn send(msg: Message) -> bool {
    QUEUE.lock().push_back(msg).is_ok()
}

/// Receive the next message if available.
pub fn recv() -> Option<Message> {
    QUEUE.lock().pop_front()
}
