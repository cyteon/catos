use crate::drivers::io::{inb, outb};

const PIT_FREQ: u32 = 1193182;
pub const TICK_HZ: u32 = 100;

pub fn init() {
    let divisor = (PIT_FREQ / TICK_HZ) as u16;
    outb(0x43, 0x36);
    outb(0x40, (divisor & 0xFF) as u8);
    outb(0x40, (divisor >> 8) as u8);

    while inb(0x64) & 1 != 0 {
        let _ = inb(0x60);
    }
}
