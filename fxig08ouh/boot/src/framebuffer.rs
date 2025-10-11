use core::fmt::Write;
use core::panic::PanicInfo;

use mk::{FrameBufferInfo, PixelFormat};
use spin::Mutex;
use uart_16550::SerialPort;
use uefi::proto::console::gop::{FrameBuffer, GraphicsOutput, PixelFormat as UefiPixelFormat};

static EARLY_FB: Mutex<Option<BootFrameBuffer>> = Mutex::new(None);
static SERIAL: Mutex<Option<SerialPort>> = Mutex::new(None);

/// Framebuffer description captured from UEFI GOP.
#[derive(Clone, Copy)]
pub struct BootFrameBuffer {
    pub base: *mut u8,
    pub stride: usize,
    pub info: FrameBufferInfo,
}

impl BootFrameBuffer {
    pub fn new(framebuffer: &FrameBuffer, gop: &GraphicsOutput) -> Self {
        let mode = gop.current_mode_info();
        let format = match mode.pixel_format() {
            UefiPixelFormat::Rgb => PixelFormat::Rgb,
            UefiPixelFormat::Bgr => PixelFormat::Bgr,
            UefiPixelFormat::Bitmask => PixelFormat::Bitmask,
            UefiPixelFormat::BltOnly => PixelFormat::BltOnly,
        };

        Self {
            base: framebuffer.as_mut_ptr(),
            stride: mode.stride(),
            info: FrameBufferInfo {
                width: mode.resolution().0,
                height: mode.resolution().1,
                pixel_format: format,
                size: framebuffer.size(),
                phys_addr: framebuffer.as_mut_ptr() as u64,
            },
        }
    }
}

/// Install the framebuffer for later write attempts (panic path, logging).
pub fn install(fb: BootFrameBuffer) {
    *EARLY_FB.lock() = Some(fb);
}

/// Attempt to set up legacy serial logging early; optional.
pub fn init_serial(port_address: u16) {
    let mut serial = unsafe { SerialPort::new(port_address) };
    serial.init();
    *SERIAL.lock() = Some(serial);
}

/// Draw the boot banner so operators know we made it into the loader.
pub fn draw_banner() {
    if let Some(mut writer) = framebuffer_console() {
        let _ = writeln!(writer, "AegisOS fxig08ouh (UEFI)");
    }
    if let Some(serial) = &mut *SERIAL.lock() {
        let _ = writeln!(serial, "AegisOS fxig08ouh (UEFI)");
    }
}

/// Print panic info to both framebuffer and serial.
pub fn panic_flush(info: &PanicInfo) {
    if let Some(mut writer) = framebuffer_console() {
        let _ = writeln!(writer, "panic: {info}");
    }
    if let Some(serial) = &mut *SERIAL.lock() {
        let _ = writeln!(serial, "panic: {info}");
    }
}

fn framebuffer_console() -> Option<FrameBufferWriter> {
    EARLY_FB
        .lock()
        .as_ref()
        .copied()
        .map(|fb| FrameBufferWriter { fb })
}

/// Minimal framebuffer writer that blits ASCII glyphs as 8x16 rectangles.
pub struct FrameBufferWriter {
    fb: BootFrameBuffer,
}

impl Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let BootFrameBuffer { base, stride, info } = self.fb;
        let mut cursor_x = 8usize;
        let mut cursor_y = 8usize;
        for ch in s.chars() {
            if ch == '\n' {
                cursor_y = cursor_y.saturating_add(16);
                cursor_x = 8;
                continue;
            }
            unsafe {
                blit_glyph(base, stride, info.width, info.height, cursor_x, cursor_y, ch as u8);
            }
            cursor_x = cursor_x.saturating_add(8);
        }
        Ok(())
    }
}

/// SAFETY: caller must ensure coordinates and framebuffer are valid.
unsafe fn blit_glyph(
    base: *mut u8,
    stride: usize,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    ch: u8,
) {
    if x >= width || y >= height {
        return;
    }
    let stride_bytes = stride * 4;
    let rect_w = 8;
    let rect_h = 16;
    for row in 0..rect_h {
        if y + row >= height {
            break;
        }
        let row_ptr = base.add((y + row) * stride_bytes + x * 4);
        for col in 0..rect_w {
            if x + col >= width {
                break;
            }
            let px = row_ptr.add(col * 4);
            let color = 0x30 + (ch % 200);
            px.write_volatile(color); // B
            px.add(1).write_volatile(color); // G
            px.add(2).write_volatile(0xff); // R
            px.add(3).write_volatile(0xff); // Alpha (ignored)
        }
    }
}

/// Retrieve the framebuffer descriptor if it has been installed during boot.
pub fn current() -> Option<BootFrameBuffer> {
    EARLY_FB.lock().as_ref().copied()
}
