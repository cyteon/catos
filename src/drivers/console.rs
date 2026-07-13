use core::{
    fmt,
    sync::atomic::{AtomicU32, Ordering},
};

use limine::framebuffer::Framebuffer;
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

use crate::{
    drivers::fb,
    lib::font::{FONT, Font, parse_font},
};

enum Ansi {
    Normal,
    Esc,
    Csi(u32),
}

pub struct Console {
    font: Font,
    col: usize,
    row: usize,
    ansi: Ansi,
    cursor_visible: bool,
}

unsafe impl Send for Console {}

pub static COLOR: AtomicU32 = AtomicU32::new(0xFFFFFF);

const MARGIN: usize = 8;

impl Console {
    fn cols(&self) -> usize {
        (fb::info().width - MARGIN * 2) / self.font.width
    }

    fn rows(&self) -> usize {
        (fb::info().height - MARGIN * 2) / self.font.height
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        unsafe {
            let offset = y * fb::info().pitch + x * 4;
            fb::info()
                .addr
                .add(offset)
                .cast::<u32>()
                .write_volatile(color);
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
                let color = if on {
                    COLOR.load(Ordering::Relaxed)
                } else {
                    0x000000
                };

                self.set_pixel(px + x, py + y, color);
            }
        }
    }

    fn scroll(&mut self) {
        let line = self.font.height * fb::info().pitch;
        let top = MARGIN * fb::info().pitch;
        let band = self.rows() * self.font.height * fb::info().pitch;

        unsafe {
            core::ptr::copy(
                fb::info().addr.add(top + line),
                fb::info().addr.add(top),
                band - line,
            );
            core::ptr::write_bytes(fb::info().addr.add(top + band - line), 0, line);
        }
    }

    fn newline(&mut self) {
        self.draw_cursor(false);

        self.col = 0;
        self.row += 1;

        if self.row >= self.rows() {
            self.scroll();
            self.row -= 1;
        }
    }

    fn backspace(&mut self) {
        if self.col == 0 {
            return;
        }

        self.draw_cursor(false);

        self.col -= 1;
        self.draw_char(' ');
    }

    pub fn put_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),

            '\x08' => self.backspace(),

            '\t' => {
                for _ in 0..4 {
                    self.put_char(' ');
                }
            }

            _ => {
                if self.col >= self.cols() {
                    self.newline();
                }

                self.draw_char(c);
                self.col += 1;
            }
        }
    }

    fn sgr(&mut self, n: u32) {
        let color = match n {
            0 => 0xFFFFFF,
            30 => 0x000000,
            31 => 0xFF0000,
            32 => 0x00FF00,
            33 => 0xFFFF00,
            34 => 0x0000FF,
            35 => 0xFF00FF,
            36 => 0x00FFFF,
            37 => 0xFFFFFF,
            _ => return,
        };

        COLOR.store(color, Ordering::Relaxed);
    }

    fn draw_cursor(&mut self, on: bool) {
        let px = MARGIN + self.col * self.font.width;
        let py = MARGIN + self.row * self.font.height;
        let color = if on {
            COLOR.load(Ordering::Relaxed)
        } else {
            0x000000
        };

        for y in 0..self.font.height {
            for x in 0..self.font.width {
                self.set_pixel(px + x, py + y, color);
            }
        }
    }

    pub fn clear(&mut self) {
        let info = fb::info();

        unsafe {
            core::ptr::write_bytes(info.addr, 0, info.height * info.pitch);
        }

        self.col = 0;
        self.row = 0;
        self.cursor_visible = false;
        self.ansi = Ansi::Normal;
        COLOR.store(0xFFFFFF, Ordering::Relaxed);
    }
}

pub static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

pub fn init() {
    let font = parse_font();

    crate::println!(
        "font: {}x{} bpg={} start={}",
        font.width,
        font.height,
        font.bpg,
        font.glyph_start
    );

    *CONSOLE.lock() = Some(Console {
        font: font,
        col: 0,
        row: 0,
        ansi: Ansi::Normal,
        cursor_visible: false,
    });
}

impl fmt::Write for Console {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for char in string.chars() {
            match self.ansi {
                Ansi::Normal => match char {
                    '\x1b' => self.ansi = Ansi::Esc,
                    _ => self.put_char(char),
                },

                Ansi::Esc => {
                    self.ansi = if char == '[' {
                        Ansi::Csi(0)
                    } else {
                        Ansi::Normal
                    };
                }

                Ansi::Csi(n) => match char {
                    '0'..='9' => self.ansi = Ansi::Csi(n * 10 + (char as u32 - '0' as u32)),
                    ';' => self.ansi = Ansi::Csi(0),
                    'm' => {
                        self.sgr(n);
                        self.ansi = Ansi::Normal;
                    }

                    _ => self.ansi = Ansi::Normal,
                },
            }
        }

        Ok(())
    }
}

pub fn tick_cursor() {
    without_interrupts(|| {
        if fb::OWNER.load(Ordering::Relaxed) != 0 {
            return;
        }

        if let Some(c) = CONSOLE.lock().as_mut() {
            c.cursor_visible = !c.cursor_visible;
            c.draw_cursor(c.cursor_visible);
        }
    })
}

pub fn clear() {
    without_interrupts(|| {
        if fb::OWNER.load(Ordering::Relaxed) != 0 {
            return;
        }

        if let Some(c) = CONSOLE.lock().as_mut() {
            c.clear();
        }
    })
}

pub fn _print(args: fmt::Arguments) {
    without_interrupts(|| {
        use fmt::Write;

        super::serial::_print(args);

        if fb::OWNER.load(Ordering::Relaxed) != 0 {
            return;
        }

        if let Some(c) = CONSOLE.lock().as_mut() {
            let _ = c.write_fmt(args);
        }
    })
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
        crate::print!("\n");
    }};

    ($($arg:tt)*) => {{
        crate::print!("{}\n", format_args!($($arg)*));
    }}
}

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";

pub const RESET: &str = "\x1b[0m";
