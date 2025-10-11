#![no_std]

pub mod checkpoint;
pub mod profiles;

pub fn init() {
    profiles::install_defaults();
}
