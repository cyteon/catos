#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use limine::{
    BaseRevision,
    request::{FramebufferRequest, RequestsEndMarker, RequestsStartMarker},
};

use crate::drivers::console::{GREEN, RED, RESET};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END: RequestsEndMarker = RequestsEndMarker::new();

pub mod drivers;
pub mod lib;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    drivers::serial::init();
    println!();
    println!("[OK] serial driver initialized");

    assert!(BASE_REVISION.is_supported());
    println!("limine base rev ok");

    let framebuffer = FRAMEBUFFER
        .get_response()
        .expect("no framebuffer")
        .framebuffers()
        .next()
        .expect("no framebuffer");

    println!(
        "framebuffer: {}x{} @ {} bpp",
        framebuffer.width(),
        framebuffer.height(),
        framebuffer.bpp()
    );

    drivers::console::init(&framebuffer);
    println!("[ {}OK{} ] console driver initialized", GREEN, RESET);

    lib::gdt::init();
    println!("[ {}OK{} ] gdt loaded", GREEN, RESET);

    lib::idt::init();
    println!("[ {}OK{} ] idt loaded", GREEN, RESET);

    x86_64::instructions::interrupts::enable();
    println!("[ {}OK{} ] interrupts enabled", GREEN, RESET);

    drivers::pic::init();
    println!("[ {}OK{} ] pic initialized", GREEN, RESET);

    hlt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!(
        "\n[ {}PANIC{} ] paniced at {}",
        RED,
        RESET,
        _info.location().unwrap()
    );
    println!("{}{}\n", _info.message(), RESET);

    hlt()
}

fn hlt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
