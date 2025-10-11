# gfx crate

Graphics primitives for fxig08ouh:
- Framebuffer compositor orchestrating windows/cursor drawing.
- Minimal bitmap font for text blitting.
- Geometry helpers for rectangle fills and overlay support.

## TODOs
- Replace immediate-mode drawing with surface management and dirty rect tracking.
- Support alpha blending for cursor overlay.
- Integrate VM overlay mapping once virtiofs path lands.
