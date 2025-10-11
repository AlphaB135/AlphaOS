# mk crate

Microkernel core for fxig08ouh. This crate owns:
- Boot-time entry (`init`) that consumes `BootInfo` from the UEFI loader.
- Memory management: frame allocator, paging tables, higher-half mapping, heap bootstrap.
- Capability-based micro-IPC primitives.
- Scheduler, timers (HPET + APIC), and syscall dispatch layer.
- Architecture-specific interrupt, GDT/IDT, and CPU setup.

## TODOs (Week 1-2)
- Wire the frame allocator to real UEFI descriptors and feed paging builder.
- Implement HPET programming and APIC timer calibration.
- Flesh out the IPC queue with wait/wake semantics and cross-core support.
- Round-robin scheduler should support sleeping tasks and capacity-aware run queues.

Unsafe code appears only inside architecture bring-up (`arch/`), paging, and direct register access; every `unsafe` block documents invariants.
