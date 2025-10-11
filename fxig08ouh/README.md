# AegisOS fxig08ouh

> Rust-first, non-Linux, micro-hypervisor + microkernel OS targeting x86_64 UEFI.

## Vision
- Deliver a clean-room, capability-secure microkernel running on top of a bespoke Rust micro-hypervisor.
- Boot exclusively via UEFI and sustain a measured chain without reusing Linux or GPL kernel code.
- Provide a modern, minimal GUI path from firmware hand-off to user experience within two weeks.
- Keep the codebase modular: every subsystem in its own crate with clear ownership and explicit capability interfaces.

### Non-Linux Constraint
This repository intentionally avoids importing Linux kernel sources or assuming Linux semantics. All abstractions, memory managers, device drivers, and servers are authored from scratch with Rust `#![no_std]` first, leaning on UEFI specifications, ACPI tables, and PCI standards.

## Quick Start
1. Install the Rust toolchain described in `rust-toolchain.toml` (`rustup toolchain install nightly --component rust-src`).
2. Fetch build helpers: `cargo install just cargo-binutils` and ensure `lld` is available.
3. Obtain OVMF firmware (see `scripts/build-ovmf.md`).
4. Build everything: `just build` (or `cargo build -Zbuild-std=core,alloc --target x86_64-unknown-none`).
5. Run the Week-1/2 demo in QEMU: `scripts/run-qemu.sh --ovmf /path/to/OVMF_CODE.fd`.
6. Regenerate the EFI boot image: `scripts/gen-iso.sh target/ovmf.img`.

### QEMU Flags
The provided run script launches:
```
qemu-system-x86_64 \
  -machine q35,accel=kvm:tcg \
  -cpu max,+vmx,+svm \
  -m 2G \
  -drive if=pflash,format=raw,readonly=on,file=$OVMF_CODE \
  -drive if=pflash,format=raw,file=$OVMF_VARS \
  -device nvme,drive=nvme0,serial=FXIG08OUHNVME \
  -drive id=nvme0,file=${NVME_DISK},format=raw,if=none \
  -netdev user,id=net0,hostfwd=tcp::2222-:22 \
  -device virtio-net-pci,netdev=net0 \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  -display sdl,gl=on
```
Attach GDB via `-s -S` flags by passing `--gdb` to the script.

## Security Posture
- High-assurance boot chain: TPM measurements and reproducible build steps (`SECURITY.md`).
- Capability-first design; every server runs with minimal privileges once manifests are enforced.
- Unsafe Rust is restricted to audited leaf modules with `// SAFETY:` documentation.
- Coordinated disclosure policy and hard dependency pinning tracked in `SECURITY.md`.

## Roadmap (Week 1-2 MVP)
- [ ] Boot via UEFI, gather memory/ACPI/FB, enter long mode, print banner
- [ ] Initialize microkernel memory manager (paging + heap)
- [ ] Install GDT/IDT, enable APIC + HPET tick
- [ ] Provide framebuffer GUI with compositor + cursor
- [ ] Establish micro-IPC primitives and capabilities
- [ ] Identify and read blocks from NVMe device
- [ ] Bring up virtio-net skeleton with descriptor logging
- [ ] Instantiate VM skeleton (VMCS/SVM structs, control MSRs)
- [ ] POSIX shim exporting syscall numbers and hello example
- [ ] Scripts: ISO build, QEMU launch, CI smoke test

## Contribution Guide
### Rust Style & Tooling
- Format with `cargo fmt` (nightly) and lint via `cargo clippy --no-deps`.
- Keep `#![no_std]` surfaces clean; isolate host-side utilities behind `cfg(test)` and `cfg(feature = "std").`
- Prefer `Result<T, Error>` over panics in non-init code. Document any `panic!` with a justification comment.
- Document every public type and function with rustdoc. Each `unsafe` block requires a preceding `// SAFETY:` comment describing invariants.

### Unsafe Code Discipline
1. Minimize `unsafe`; wrap hardware calls in tightly scoped modules.
2. State invariants, required preconditions, and postconditions inline.
3. Add unit or integration tests for safe wrappers where feasible using host simulators or QEMU.

### Pull Requests
- Include a short design rationale in PR description.
- Add or update Week-1/2 roadmap checkboxes.
- Ensure CI passes (`cargo fmt`, `cargo clippy`, `just build`, `just qemu-smoke`).
- For hardware touching code, reference spec sections in comments for future audit.

### Testing Expectations
- Provide host-side unit tests for pure algorithms (e.g., capability tables, message queues).
- Use QEMU scripts for integration; CI smoke test prints `AegisOS fxig08ouh ready` to serial.

## Licensing & Attribution
This project is original work. Any third-party crates must be compatible with dual MIT/Apache licensing. No Linux kernel code is imported.
