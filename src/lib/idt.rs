use core::sync::atomic::Ordering;

use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, KeyState, Keyboard, ScancodeSet1, layouts};
use spin::mutex::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{
    drivers::{
        console::{RED, RESET},
        fb, io, pic, serial,
    },
    lib::{
        keys::{self, Key},
        rawkeys::{self, RawKey},
        tasks,
    },
};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt[32].set_handler_fn(timer_handler);
        idt[33].set_handler_fn(keyboard_handler);
        idt[36].set_handler_fn(serial_in_handler);

        idt
    };
}

pub fn init() {
    IDT.load()
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    fb::force_console();

    crate::println!("\n[ {}BREAKPOINT{} ]", RED, RESET);
    crate::println!("{:#?}\n", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    fb::force_console();

    crate::println!(
        "\n[ {}DOUBLE FAULT{} ] error code: {}",
        RED,
        RESET,
        error_code
    );
    crate::println!("{:#?}\n", stack_frame);

    crate::hlt()
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    fb::force_console();

    crate::println!("\n[ {}PAGE FAULT{} ] at {:?}", RED, RESET, Cr2::read());
    crate::println!("error code: {:?}", error_code);
    crate::println!("{:#?}\n", stack_frame);

    crate::hlt()
}

extern "x86-interrupt" fn timer_handler(_frame: InterruptStackFrame) {
    crate::TICKS.fetch_add(1, Ordering::Relaxed);
    pic::end_of_interrupt(0);

    tasks::schedule();
}

static KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
    ScancodeSet1::new(),
    layouts::Us104Key,
    HandleControl::MapLettersToUnicode,
));

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    let scancode = io::inb(0x60);

    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(event)) = keyboard.add_byte(scancode) {
        if fb::OWNER.load(Ordering::Relaxed) != 0 {
            let raw = match event.state {
                KeyState::Down => RawKey::Press(event.code),
                KeyState::Up => RawKey::Release(event.code),
                KeyState::SingleShot => RawKey::Press(event.code),
            };

            rawkeys::push_raw(raw);
        } else {
            if let Some(key) = keyboard.process_keyevent(event) {
                match key {
                    DecodedKey::Unicode(char) => keys::push_key(Key::Char(char)),
                    DecodedKey::RawKey(key) => match key {
                        KeyCode::ArrowUp => keys::push_key(Key::Up),
                        KeyCode::ArrowDown => keys::push_key(Key::Down),
                        KeyCode::ArrowLeft => keys::push_key(Key::Left),
                        KeyCode::ArrowRight => keys::push_key(Key::Right),
                        _ => {}
                    },
                }

                tasks::wake(0);
            }
        }
    }

    pic::end_of_interrupt(1);
}

extern "x86-interrupt" fn serial_in_handler(_frame: InterruptStackFrame) {
    while io::inb(serial::COM1 + 5) & 1 != 0 {
        let byte = io::inb(serial::COM1);

        match byte {
            b'\r' => super::keys::push_key(Key::Char('\n')),
            b'\x7f' => super::keys::push_key(Key::Char('\x08')),
            _ => super::keys::push_key(Key::Char(byte as char)),
        }

        tasks::wake(0);
    }

    pic::end_of_interrupt(4);
}
