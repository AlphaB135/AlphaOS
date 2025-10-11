# vm crate

Virtual machine manager for hosted guests. Early focus:
- Create vCPU structures and populate VMCS/VMCB with launch state.
- Prepare control MSRs and EPT/NPT placeholders.
- Expose skeleton APIs for virtiofs and overlay integration with the compositor.

## TODOs
- Flesh out VCPU lifecycle and run loop.
- Map virtio devices into guest address space.
- Wire hypercalls into capability system.
