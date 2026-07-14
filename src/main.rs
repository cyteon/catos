#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::{panic::PanicInfo, sync::atomic::AtomicU64};

use alloc::vec::Vec;
use limine::{
    BaseRevision,
    memory_map::EntryType,
    request::{
        FramebufferRequest, HhdmRequest, MemoryMapRequest, ModuleRequest, RequestsEndMarker,
        RequestsStartMarker,
    },
};

use crate::{
    drivers::console::{GREEN, RED, RESET},
    lib::memory::STACK_TOP,
};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MODULES: ModuleRequest = ModuleRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END: RequestsEndMarker = RequestsEndMarker::new();

pub mod doom;
pub mod drivers;
pub mod lib;
mod shell;

pub static TICKS: AtomicU64 = AtomicU64::new(0);

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

    drivers::fb::init(framebuffer);
    println!("[ {}OK{} ] framebuffer initialized", GREEN, RESET);

    drivers::console::init();
    println!("[ {}OK{} ] console driver initialized", GREEN, RESET);

    lib::sse::enable();
    println!("[ {}OK{} ] sse enabled", GREEN, RESET);

    println!("[ {}OK{} ] initalizing heap", GREEN, RESET);

    let hhdm = HHDM.get_response().expect("no hhdm").offset();
    let memory_map = MEMORY_MAP.get_response().expect("no memory map");

    lib::memory::init(hhdm, memory_map.entries());
    println!("[ {}OK{} ] heap initialized", GREEN, RESET);

    let mut total_usable_memory = 0;

    for entry in memory_map.entries() {
        if entry.entry_type == EntryType::USABLE {
            total_usable_memory += entry.length;
        }
    }

    println!(
        "total usable memory: {} MiB",
        total_usable_memory / 1024 / 1024,
    );

    // test if allocation works
    let test: Vec<i64> = (0..10000).collect();
    drop(test);

    unsafe {
        core::arch::asm!(
            "mov rsp, {stack}",
            "jmp {main}",
            stack = in(reg) STACK_TOP,
            main = sym main,
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    println!("[ {}OK{} ] moved from _start into main", GREEN, RESET);

    lib::gdt::init();
    println!("[ {}OK{} ] gdt loaded", GREEN, RESET);

    lib::idt::init();
    println!("[ {}OK{} ] idt loaded", GREEN, RESET);

    drivers::pic::init();
    println!("[ {}OK{} ] pic initialized", GREEN, RESET);

    drivers::pit::init();
    println!("[ {}OK{} ] pit initialized", GREEN, RESET);

    x86_64::instructions::interrupts::enable();
    println!("[ {}OK{} ] interrupts enabled", GREEN, RESET);

    let module = MODULES
        .get_response()
        .expect("no modules")
        .modules()
        .first()
        .expect("no initrd");

    let initrd: &'static [u8] =
        unsafe { core::slice::from_raw_parts(module.addr(), module.size() as usize) };

    lib::fs::init(initrd);
    println!("[ {}OK{} ] fs initialized", GREEN, RESET);

    println!("[ {}OK{} ] boot complete", GREEN, RESET);
    println!("[ {}OK{} ] starting shell\n", GREEN, RESET);

    lib::tasks::init();

    shell::shell_loop();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        println!("\n[ {}PANIC{} ] panicked at {}", RED, RESET, location);
    } else {
        println!("\n[ {}PANIC{} ]", RED, RESET);
    }

    println!("{}{}\n", _info.message(), RESET);

    hlt()
}

fn hlt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
