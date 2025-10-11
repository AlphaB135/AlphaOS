# fs-server

Prototype storage server providing NVMe block access and an in-memory KV store for Week 1-2.

## Features
- Initializes NVMe driver (`drivers::nvme`) and performs Identify/Read commands.
- Exposes a micro-IPC endpoint for block read passthroughs and KV operations.
- Logs hexdumps of read sectors for smoke testing.

## TODOs
- Implement journaling and multi-queue NVMe support.
- Persist KV data into NVMe namespaces once write path is ready.
