//! Round-robin scheduler stub with a single core run queue.

use heapless::Deque;
use spin::Mutex;

const MAX_TASKS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId(pub u32);

static RUN_QUEUE: Mutex<Deque<TaskId, MAX_TASKS>> = Mutex::new(Deque::new());

pub fn init() {
    RUN_QUEUE.lock().clear();
    // Insert the idle task placeholder.
    let _ = RUN_QUEUE.lock().push_back(TaskId(0));
}

pub fn enqueue(task: TaskId) {
    let _ = RUN_QUEUE.lock().push_back(task);
}

pub fn tick() {
    let mut queue = RUN_QUEUE.lock();
    if let Some(task) = queue.pop_front() {
        queue.push_back(task).ok();
    }
}
