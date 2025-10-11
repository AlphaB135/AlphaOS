# Security Policy

AegisOS fxig08ouh treats security as a first-class deliverable even while operating in the open.

## Supported Versions
- `main` branch: actively maintained, receives security fixes.
- Tagged milestones (`week1-mvp`, etc.) are supported until the next milestone is published.

## Reporting Vulnerabilities
- Email: `security@aegisos.local` (PGP key to be published).
- Expected response time: 48 hours acknowledgement, 7 calendar days initial fix plan.
- Please provide reproduction steps, affected commit hash, and suggested mitigations if available.

## Coordinated Disclosure
1. Reporter contacts security team privately.
2. Team triages and prepares patch + regression tests.
3. Maintainers coordinate advisory publication (typically within 30 days).
4. Fix merges to `main` with CI proofs and reproducible build artifacts.

## High-Assurance Requirements
- All `unsafe` blocks require documented invariants and code review sign-off.
- Boot chain must remain measured (TPM extend) and reproducible from source.
- Critical cryptography must use formally verified or industry-standard libraries.

## Reproducible Builds
- CI emits deterministic artifacts (ISO, EFI binaries) using pinned toolchains.
- `scripts/gen-iso.sh` records command-line invocations; future work will include `make verify-build` that checks hashes across builders.

## Dependency Policy
- Only MIT/Apache compatible crates.
- All direct dependencies pinned via `Cargo.lock` (to be checked in once core build stabilizes).
- Third-party crates are scanned via `cargo audit` in CI (forthcoming task).

## Threat Model Overview
- Adversaries may control guest VMs, user-level tasks, and external network inputs.
- Firmware is assumed to be authentic but may expose misconfigured ACPI tables.
- DMA-capable peripherals are treated as untrusted until IOMMU isolation is enforced.
- Physical attackers are out of scope for Week-1/2 but considered for radiation/RT deployments.

## Contact
For general questions: `dev@aegisos.local`
For urgent security matters: `security@aegisos.local`
