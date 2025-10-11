#![no_std]

pub mod redundancy;
pub mod scrubber;
pub mod watchdog;

pub fn init() {
    scrubber::schedule();
}
