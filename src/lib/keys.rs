use x86_64::instructions::interrupts::without_interrupts;

#[derive(Copy, Clone)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
}

struct KeyBuffer {
    buffer: [Key; 256],
    head: usize,
    tail: usize,
}

static mut KEYS: KeyBuffer = KeyBuffer {
    buffer: [Key::Char('\0'); 256],
    head: 0,
    tail: 0,
};

pub fn push_key(key: Key) {
    unsafe {
        let next = (KEYS.head + 1) % 256;

        if next != KEYS.tail {
            let index = KEYS.head;
            KEYS.buffer[index] = key;
            KEYS.head = next;
        }
    }
}

pub fn pop_key() -> Option<Key> {
    without_interrupts(|| unsafe {
        if KEYS.tail == KEYS.head {
            None
        } else {
            let index = KEYS.tail;
            let char = KEYS.buffer[index];
            KEYS.tail = (KEYS.tail + 1) % 256;
            Some(char)
        }
    })
}
