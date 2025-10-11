# boot crate

UEFI entrypoint for fxig08ouh. Responsibilities:
- Entered by firmware, capture the system table, memory map, framebuffer, and ACPI pointers.
- Initialize an early heap for allocator bootstrap, configure paging, and switch to long mode.
- Draw the boot banner (`"AegisOS fxig08ouh (UEFI)"`) on the framebuffer for operator feedback.
- Prepare a `mk::bootinfo::BootInfo` payload and hand off control to the microkernel (`mk::init`).
- Provide logging via serial (16550) and framebuffer text blits for fatal panics.

## TODOs
- Wire HPET discovery inside the boot phase for timer calibration.
- Implement measured boot events and TPM extend operations.
- Harden memory map parsing (guard against overlapping or malformed descriptors).

This crate builds as a `cdylib` suitable for UEFI (`BOOTX64.EFI`).
