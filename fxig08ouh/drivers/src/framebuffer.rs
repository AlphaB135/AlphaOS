//! Helpers around the kernel framebuffer, reused by compositor and logging layers.

use mk::bootinfo::FrameBufferInfo;

pub struct FrameBuffer {
    pub ptr: *mut u8,
    pub info: FrameBufferInfo,
}

impl FrameBuffer {
    /// SAFETY: caller must ensure the framebuffer pointer is valid and mapped.
    pub unsafe fn write_pixel(&self, x: usize, y: usize, color: [u8; 4]) {
        let stride_bytes = self.info.width * 4;
        let offset = y * stride_bytes + x * 4;
        let pixel = self.ptr.add(offset);
        pixel.write_volatile(color[0]);
        pixel.add(1).write_volatile(color[1]);
        pixel.add(2).write_volatile(color[2]);
        pixel.add(3).write_volatile(color[3]);
    }
}
