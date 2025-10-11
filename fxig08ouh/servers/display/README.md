# display-server

A proto userspace display compositor that owns the framebuffer during Week 1-2.
- Receives draw commands over micro-IPC (`Display.DrawRect`, `Display.BlitText`, `Display.SetCursor`).
- Uses `gfx` crate primitives to render windows, titles, and a cursor.
- Logs keyboard/mouse events forwarded from `servers/input`.

## TODOs
- Replace busy loop with event-driven wakeups from the scheduler.
- Support double buffering and vsync pacing.
- Introduce surface handles for VM overlay integration.
