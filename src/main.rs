#![no_std]
#![no_main]

use core::panic::PanicInfo;

use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker, request::FramebufferRequest};

#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(link_section = ".requests")]
static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[unsafe(link_section = ".requests_start_marker")]
static _START: RequestsStartMarker = RequestsStartMarker::new();

#[unsafe(link_section = ".requests_end_marker")]
static _END: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    hlt()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt()
}

fn hlt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
