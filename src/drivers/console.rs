use core::fmt;

use limine::framebuffer::Framebuffer;
use spin::Mutex;

use crate::lib::font::{FONT, Font, parse_font};

pub struct Console {
    addr: *mut u8,
    pitch: usize,
    width: usize,
    height: usize,
    font: Font,
    col: usize,
    row: usize,
}

unsafe impl Send for Console {}

const MARGIN: usize = 8;

impl Console {
    fn cols(&self) -> usize {
        (self.width - MARGIN * 2) / self.font.width
    }

    fn rows(&self) -> usize {
        (self.height - MARGIN * 2) / self.font.height
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        unsafe {
            let offset = y * self.pitch + x * 4;
            self.addr.add(offset).cast::<u32>().write_volatile(color);
        }
    }

    fn draw_char(&mut self, char: char) {
        let start = self.font.glyph_start + char as usize * self.font.bpg;
        let glyph = &FONT[start..(start + self.font.bpg)];

        let bpr = (self.font.width + 7) / 8;

        let px = MARGIN + self.col * self.font.width;
        let py = MARGIN + self.row * self.font.height;

        for y in 0..self.font.height {
            for x in 0..self.font.width {
                let byte = glyph[y * bpr + x / 8];
                let on = byte & (0x80 >> (x % 8)) != 0;
                let color = if on { 0xFFFFFF } else { 0x000000 };

                self.set_pixel(px + x, py + y, color);
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
    }

    pub fn put_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            _ => {
                if self.col >= self.cols() {
                    self.newline();
                }

                self.draw_char(c);
                self.col += 1;
            }
        }
    }
}

pub static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

pub fn init(framebuffer: &Framebuffer) {
    let font = parse_font();

    crate::serial_println!(
        "font: {}x{} bpg={} start={}",
        font.width,
        font.height,
        font.bpg,
        font.glyph_start
    );

    *CONSOLE.lock() = Some(Console {
        addr: framebuffer.addr(),
        pitch: framebuffer.pitch() as usize,
        width: framebuffer.width() as usize,
        height: framebuffer.height() as usize,
        font: font,
        col: 0,
        row: 0,
    });
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for char in string.chars() {
            self.put_char(char);
        }

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    if let Some(c) = CONSOLE.lock().as_mut() {
        let _ = c.write_fmt(args);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        crate::drivers::console::_print(format_args!($($arg)*));
    }}
}

#[macro_export]
macro_rules! println {
    () => {{
        crate::serial_print!("\n");
    }};

    ($($arg:tt)*) => {{
        crate::print!("{}\n", format_args!($($arg)*));
    }}
}
