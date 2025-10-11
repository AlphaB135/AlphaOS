//! VM manager skeleton structures.

#[derive(Default)]
pub struct Vmcs {
    pub revision_id: u32,
    pub abort_indicator: u32,
}

#[derive(Default)]
pub struct Vcpu {
    pub id: u16,
    pub vmcs: Vmcs,
}

static mut VCPU: Vcpu = Vcpu {
    id: 0,
    vmcs: Vmcs {
        revision_id: 0,
        abort_indicator: 0,
    },
};

pub fn bootstrap() {
    unsafe {
        VCPU = Vcpu::default();
    }
}

pub fn vcpu() -> &'static Vcpu {
    unsafe { &VCPU }
}
