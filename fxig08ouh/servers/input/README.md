# input-server

Translates PS/2 and USB HID events into compositor-friendly IPC messages.

## Week 1-2 Targets
- Poll PS/2 keyboard controller via `drivers::ps2`.
- Convert scan codes into key events and forward to display server queue.
- Stub USB HID discovery until xHCI driver arrives.
