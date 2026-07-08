#![no_std]
#![no_main]

use core::panic::PanicInfo;

use limine::{
    BaseRevision,
    request::{FramebufferRequest, RequestsEndMarker, RequestsStartMarker},
};

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
    serial_println!();
    serial_println!("serial driver initialized");

    assert!(BASE_REVISION.is_supported());
    serial_println!("limine base rev ok");

    serial_println!(
        "framebuffer is some?: {:?}",
        FRAMEBUFFER.get_response().is_some()
    );

    hlt()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("\n\x1b[31m--- PANIC ---");
    serial_println!("{}\n\x1b[0m", _info);

    hlt()
}

fn hlt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
