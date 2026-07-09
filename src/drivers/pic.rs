use crate::drivers::io::{io_wait, outb};

const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;

const PIC1_COMMAND: u16 = PIC1;
const PIC1_DATA: u16 = PIC1 + 1;

const PIC2_COMMAND: u16 = PIC2;
const PIC2_DATA: u16 = PIC2 + 1;

const PIC1_OFFSET: u8 = 32;
const PIC2_OFFSET: u8 = 40;

pub fn init() {
    outb(PIC1_COMMAND, 0x11);
    io_wait();

    outb(PIC2_COMMAND, 0x11);
    io_wait();

    outb(PIC1_DATA, PIC1_OFFSET);
    io_wait();

    outb(PIC2_DATA, PIC2_OFFSET);
    io_wait();

    outb(PIC1_DATA, 0x04);
    io_wait();

    outb(PIC2_DATA, 0x02);
    io_wait();

    outb(PIC1_DATA, 0x01);
    io_wait();

    outb(PIC2_DATA, 0x01);
    io_wait();

    outb(PIC1_DATA, 0);
    outb(PIC2_DATA, 0);
}
