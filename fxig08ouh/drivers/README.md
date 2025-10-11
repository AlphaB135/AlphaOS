# drivers crate

Collection of privileged device drivers.

- `nvme`: Identify/Read primitives for NVMe PCIe devices.
- `virtio_net`: Feature negotiation and queue setup for virtio-net.
- `ps2`: Legacy keyboard controller polling.
- `usb`: Placeholder for USB HID / xHCI integration.
- `framebuffer`: Shared helpers for drawing and logging via GOP framebuffer.

Each module exports safe wrappers where possible and keeps `unsafe` localized to MMIO register access.
