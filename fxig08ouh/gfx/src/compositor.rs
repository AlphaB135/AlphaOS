//! Immediate-mode compositor that draws directly into the firmware framebuffer.

use core::cmp::min;

use drivers::framebuffer::FrameBuffer;
use mk::{get_boot_info, FrameBufferInfo};
use spin::Mutex;

use crate::font;

static FRAMEBUFFER: Mutex<Option<FrameBuffer>> = Mutex::new(None);

pub enum DisplayCommand {
    DrawRect { x: usize, y: usize, w: usize, h: usize, color: [u8; 4] },
    BlitText { x: usize, y: usize, ch: char, color: [u8; 4] },
    SetCursor { x: usize, y: usize, shape: CursorShape },
}

#[derive(Clone, Copy)]
pub enum CursorShape {
    Arrow,
    Crosshair,
}

impl DisplayCommand {
    pub fn from_payload(payload: [u64; 4]) -> Option<Self> {
        match payload[0] {
            0 => Some(DisplayCommand::DrawRect {
                x: payload[1] as usize,
                y: payload[2] as usize,
                w: payload[3] as usize,
                h: 32,
                color: [0x50, 0x50, 0xf0, 0xff],
            }),
            1 => Some(DisplayCommand::BlitText {
                x: payload[1] as usize,
                y: payload[2] as usize,
                ch: core::char::from_u32(payload[3] as u32).unwrap_or('?'),
                color: [0xff, 0xff, 0xff, 0xff],
            }),
            2 => Some(DisplayCommand::SetCursor {
                x: payload[1] as usize,
                y: payload[2] as usize,
                shape: CursorShape::Arrow,
            }),
            _ => None,
        }
    }
}

pub fn init() {
    if let Some(info) = framebuffer_info() {
        let fb = FrameBuffer {
            ptr: info.phys_addr as *mut u8,
            info,
        };
        *FRAMEBUFFER.lock() = Some(fb);
    }
}

pub fn draw_banner_window() {
    if let Some(fb) = FRAMEBUFFER.lock().as_ref() {
        draw_rect(fb, 40, 40, 320, 200, [0x20, 0x20, 0x90, 0xff]);
        draw_string(fb, 64, 64, "AegisOS fxig08ouh ready", [0xff, 0xff, 0xff, 0xff]);
    }
}

pub fn dispatch(cmd: DisplayCommand) {
    if let Some(fb) = FRAMEBUFFER.lock().as_ref() {
        match cmd {
            DisplayCommand::DrawRect { x, y, w, h, color } => draw_rect(fb, x, y, w, h, color),
            DisplayCommand::BlitText { x, y, ch, color } => draw_char(fb, x, y, ch, color),
            DisplayCommand::SetCursor { x, y, shape: _ } => draw_cursor(fb, x, y),
        }
    }
}

fn framebuffer_info() -> Option<FrameBufferInfo> {
    get_boot_info().framebuffer
}

fn draw_rect(fb: &FrameBuffer, x: usize, y: usize, w: usize, h: usize, color: [u8; 4]) {
    let info = fb.info;
    let max_x = min(x + w, info.width);
    let max_y = min(y + h, info.height);
    for py in y..max_y {
        for px in x..max_x {
            unsafe {
                fb.write_pixel(px, py, color);
            }
        }
    }
}

fn draw_char(fb: &FrameBuffer, x: usize, y: usize, ch: char, color: [u8; 4]) {
    let glyph = font::glyph(ch);
    for (row_idx, row) in glyph.iter().enumerate() {
        for bit in 0..8 {
            if (row >> (7 - bit)) & 1 == 1 {
                unsafe { fb.write_pixel(x + bit, y + row_idx, color) };
            }
        }
    }
}

fn draw_string(fb: &FrameBuffer, x: usize, y: usize, text: &str, color: [u8; 4]) {
    for (idx, ch) in text.chars().enumerate() {
        draw_char(fb, x + idx * 8, y, ch, color);
    }
}

fn draw_cursor(fb: &FrameBuffer, x: usize, y: usize) {
    for dy in 0..16 {
        for dx in 0..16 {
            if dx == dy {
                unsafe { fb.write_pixel(x + dx, y + dy, [0xff, 0xff, 0xff, 0xff]) };
            }
        }
    }
}
