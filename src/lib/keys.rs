use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

const BUFFER_SIZE: usize = 256;

struct KeyBuffer {
    buffer: [char; BUFFER_SIZE],
    head: usize,
    tail: usize,
}

static mut KEYS: KeyBuffer = KeyBuffer {
    buffer: ['\0'; BUFFER_SIZE],
    head: 0,
    tail: 0,
};

pub fn push_key(char: char) {
    unsafe {
        let next = (KEYS.head + 1) % BUFFER_SIZE;

        if next != KEYS.tail {
            let index = KEYS.head;
            KEYS.buffer[index] = char;
            KEYS.head = next;
        }
    }
}

pub fn pop_key() -> Option<char> {
    without_interrupts(|| unsafe {
        if KEYS.tail == KEYS.head {
            None
        } else {
            let index = KEYS.tail;
            let char = KEYS.buffer[index];
            KEYS.tail = (KEYS.tail + 1) % BUFFER_SIZE;
            Some(char)
        }
    })
}
