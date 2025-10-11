# hv crate

Micro-hypervisor layer setting up VT-x (Intel) and SVM (AMD) controls, with IOMMU hooks and attestation plumbing.

## Scope
- Discover virtualization extensions, configure control MSRs, and create VMCS/VMCB templates.
- Provide safe wrappers around privileged instructions (`vmxon`, `vmclear`, `vmload`, etc.).
- Prepare IOMMU structures for device passthrough (stubbed for Week 1-2).
- Surface attestation hooks for the VM agent once the Windows guest is online.

## TODOs
- Calibrate MSR bitmasks against CPUID leafs to detect unsupported features.
- Implement nested paging structures for the guest.
- Wire attestation module to TPM measurements.
