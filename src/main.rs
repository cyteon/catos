#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
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
