use core::sync::atomic::AtomicBool;

use pc_keyboard::KeyCode;
use x86_64::instructions::interrupts::without_interrupts;

#[derive(Clone, Copy)]
pub enum RawKey {
    Press(KeyCode),
    Release(KeyCode),
}

struct RawBuffer {
    buffer: [RawKey; 256],
    head: usize,
    tail: usize,
}

static mut RAWKEYS: RawBuffer = RawBuffer {
    buffer: [RawKey::Press(KeyCode::Escape); 256],
    head: 0,
    tail: 0,
};

pub fn push_raw(key: RawKey) {
    unsafe {
        let next = (RAWKEYS.head + 1) % 256;

        if next != RAWKEYS.tail {
            RAWKEYS.buffer[RAWKEYS.head] = key;
            RAWKEYS.head = next;
        }
    }
}

pub fn pop_raw() -> Option<RawKey> {
    without_interrupts(|| unsafe {
        if RAWKEYS.tail == RAWKEYS.head {
            None
        } else {
            let key = RAWKEYS.buffer[RAWKEYS.tail];
            RAWKEYS.tail = (RAWKEYS.tail + 1) % 256;
            Some(key)
        }
    })
}
