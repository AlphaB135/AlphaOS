#![no_std]

pub mod ad;
pub mod mdm;
pub mod provisioning;

pub fn init() {
    ad::init();
    mdm::init();
}
