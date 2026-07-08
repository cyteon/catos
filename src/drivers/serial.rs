use core::fmt;

const COM1: u16 = 0x3F8;

fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

pub fn init() {
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 0x03);
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03);
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

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!(crate::drivers::serial::Serial, $($arg)*);
    }};
}

#[macro_export]
macro_rules! serial_println {
    () => {{
        crate::serial_print!("\n");
    }};

    ($($arg:tt)*) => {{
        crate::serial_print!("{}\n", format_args!($($arg)*));
    }};
}
