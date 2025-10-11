# net-server

Virtio-net proof-of-concept server. Goals for Week 1-2:
- Probe PCI for virtio-net devices and negotiate features.
- Initialize RX/TX descriptor rings via `drivers::virtio_net`.
- Log incoming descriptors and echo metadata over IPC for debug.

Longer term this server will host the network stack and virtiofs integration.
