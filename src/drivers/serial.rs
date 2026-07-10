use core::fmt;

use crate::drivers::io::outb;

pub const COM1: u16 = 0x3F8;

pub fn init() {
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 0x03);
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03);
    outb(COM1 + 1, 0x01);
    outb(COM1 + 2, 0xC7);
    outb(COM1 + 4, 0x0B);
}

fn writeb(byte: u8) {
    outb(COM1, byte);
}

pub struct Serial;

impl fmt::Write for Serial {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.bytes() {
            if byte == b'\n' {
                writeb(b'\r');
            }

            writeb(byte);
        }

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    let _ = Serial.write_fmt(args);
}
