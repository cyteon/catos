use alloc::{
    alloc::{alloc, alloc_zeroed, dealloc},
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::{alloc::Layout, sync::atomic::Ordering};
use x86_64::instructions::interrupts::without_interrupts;

use crate::lib::{
    fs,
    tasks::{self, TaskState},
};

#[unsafe(no_mangle)]
pub static mut errno: i32 = 0;

#[unsafe(no_mangle)]
pub static stdout: i32 = 1;

#[unsafe(no_mangle)]
pub static stderr: i32 = 2;

const HDR: usize = 16;

fn layout(total: usize) -> Layout {
    Layout::from_size_align(total, 16).unwrap()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    unsafe {
        let total = size + HDR;
        let ptr = alloc(layout(total));

        if ptr.is_null() {
            return core::ptr::null_mut();
        }

        (ptr as *mut usize).write(size);
        ptr.add(HDR)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    unsafe {
        if ptr.is_null() {
            return;
        }

        let base = ptr.sub(HDR);
        let size = (base as *mut usize).read();

        dealloc(base, layout(size + HDR));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(num: usize, size: usize) -> *mut u8 {
    unsafe {
        let bytes = num * size;
        let total = bytes + HDR;

        let ptr = alloc_zeroed(layout(total));

        if ptr.is_null() {
            return core::ptr::null_mut();
        }

        (ptr as *mut usize).write(bytes);
        ptr.add(HDR)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    unsafe {
        if ptr.is_null() {
            return malloc(size);
        }

        if size == 0 {
            free(ptr);
            return core::ptr::null_mut();
        }

        let old_size = (ptr.sub(HDR) as *mut usize).read();
        let new_ptr = malloc(size);

        if new_ptr.is_null() {
            return core::ptr::null_mut();
        }

        core::ptr::copy_nonoverlapping(ptr, new_ptr, old_size.min(size));
        free(ptr);

        new_ptr
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    unsafe {
        let mut n = 0;

        while *s.add(n) != 0 {
            n += 1;
        }

        n
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcmp(a: *const u8, b: *const u8) -> i32 {
    unsafe {
        let mut i = 0;

        loop {
            let ac = *a.add(i);
            let bc = *b.add(i);

            if ac != bc || ac == 0 {
                return ac as i32 - bc as i32;
            }

            i += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let ac = *a.add(i);
            let bc = *b.add(i);

            if ac != bc || ac == 0 {
                return ac as i32 - bc as i32;
            }

            if ac == 0 {
                break;
            }
        }

        0
    }
}

fn lower(c: u8) -> u8 {
    if c.is_ascii_uppercase() {
        c.to_ascii_lowercase()
    } else {
        c
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn strcasecmp(a: *const u8, b: *const u8) -> i32 {
    unsafe {
        let mut i = 0;

        loop {
            let ac = lower(*a.add(i));
            let bc = lower(*b.add(i));

            if ac != bc || ac == 0 {
                return ac as i32 - bc as i32;
            }

            i += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn strncasecmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let ac = lower(*a.add(i));
            let bc = lower(*b.add(i));

            if ac != bc || ac == 0 {
                return ac as i32 - bc as i32;
            }

            if ac == 0 {
                break;
            }
        }

        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;

        while i < n && *src.add(i) != 0 {
            *dest.add(i) = *src.add(i);
            i += 1;
        }

        while i < n {
            *dest.add(i) = 0;
            i += 1;
        }

        dest
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strrchr(s: *const u8, c: i32) -> *const u8 {
    unsafe {
        let c = c as u8;
        let mut found = core::ptr::null();
        let mut i = 0;

        loop {
            let sc = *s.add(i);

            if sc == c {
                found = s.add(i);
            }

            if sc == 0 {
                return found;
            }

            i += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strchr(hay: *const u8, needle: *const u8) -> *const u8 {
    unsafe {
        let nlen = strlen(needle);

        if nlen == 0 {
            return hay;
        }

        let hlen = strlen(hay);

        if nlen > hlen {
            return core::ptr::null();
        }

        for i in 0..=hlen - nlen {
            if strncmp(hay.add(i), needle, nlen) == 0 {
                return hay.add(i);
            }
        }

        core::ptr::null()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strdup(s: *const u8) -> *mut u8 {
    unsafe {
        let len = strlen(s);
        let new = malloc(len + 1);

        if !new.is_null() {
            core::ptr::copy_nonoverlapping(s, new, len + 1);
        }

        new
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn toupper(c: i32) -> i32 {
    let c = c as u8;

    if c.is_ascii_lowercase() {
        c.to_ascii_uppercase() as i32
    } else {
        c as i32
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn isspace(c: i32) -> i32 {
    matches!(c as u8, b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c') as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn abs(x: i32) -> i32 {
    x.wrapping_abs()
}

#[unsafe(no_mangle)]
pub extern "C" fn fabs(x: f64) -> f64 {
    if x < 0.0 { -x } else { x }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn atoi(s: *const u8) -> i32 {
    unsafe {
        let mut i = 0;

        while isspace(*s.add(i) as i32) != 0 {
            i += 1;
        }

        let mut sign = 1;

        match *s.add(i) {
            b'-' => {
                sign = -1;
                i += 1;
            }

            b'+' => {
                i += 1;
            }

            _ => {}
        }

        let mut value: i32 = 0;

        while (*s.add(i)).is_ascii_digit() {
            value = value
                .wrapping_mul(10)
                .wrapping_add((*s.add(i) - b'0') as i32);
            i += 1;
        }

        value * sign
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn atof(s: *const u8) -> f64 {
    unsafe {
        let mut i = 0;

        while isspace(*s.add(i) as i32) != 0 {
            i += 1;
        }

        let mut sign = 1.0;

        match *s.add(i) {
            b'-' => {
                sign = -1.0;
                i += 1;
            }

            b'+' => {
                i += 1;
            }

            _ => {}
        }

        let mut value: f64 = 0.0;

        while (*s.add(i)).is_ascii_digit() {
            value = value * 10.0 + ((*s.add(i) - b'0') as f64);
            i += 1;
        }

        if *s.add(i) == b'.' {
            i += 1;
            let mut frac: f64 = 0.0;

            while (*s.add(i)).is_ascii_digit() {
                value += (*s.add(i) - b'0') as f64 * frac;
                frac *= 0.1;
                i += 1;
            }
        }

        value * sign
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn system(_cmd: *const u8) -> i32 {
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn mkdir(_path: *const u8, _mode: u32) -> i32 {
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn remove(_path: *const u8) -> i32 {
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn rename(_a: *const u8, _b: *const u8) -> i32 {
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn sscanf(_s: *const u8, _fmt: *const u8) -> i32 {
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn _putchar(c: i32) {
    crate::drivers::serial::_print(format_args!("{}", c as u8 as char));
}

#[unsafe(no_mangle)]
pub extern "C" fn putchar(c: i32) -> i32 {
    _putchar(c);
    c
}

#[unsafe(no_mangle)]
pub extern "C" fn puts(s: *const u8) -> i32 {
    unsafe {
        let len = strlen(s);

        for i in 0..len {
            _putchar(*s.add(i) as i32);
        }

        _putchar(b'\n' as i32);
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn exit(_status: i32) -> ! {
    let id = tasks::CURRENT.load(Ordering::Relaxed);
    crate::drivers::fb::release_if_owner(id as u64);

    tasks::with_tasks(|tasks| tasks[id as usize].state = TaskState::Dead);

    loop {
        without_interrupts(|| {
            tasks::schedule();
        });

        x86_64::instructions::hlt();
    }
}

#[repr(C)]
pub struct FILE {
    name: String,
    data: Vec<u8>,
    pos: usize,
    write: bool,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fopen(filename: *const u8, mode: *const u8) -> *mut FILE {
    unsafe {
        let name = strdup(filename);
        if name.is_null() {
            return core::ptr::null_mut();
        }
        let name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(name, strlen(name)));

        let mode = strdup(mode);
        if mode.is_null() {
            return core::ptr::null_mut();
        }
        let mode = core::str::from_utf8_unchecked(core::slice::from_raw_parts(mode, strlen(mode)));

        let data = match fs::read(name) {
            Some(data) => data,
            None => return core::ptr::null_mut(),
        };

        crate::println!("fopen {}: {} bytes", name, data.len());

        Box::into_raw(Box::new(FILE {
            name: name.to_string(),
            data,
            pos: 0,
            write: mode.contains('w') || mode.contains('a'),
        }))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fread(ptr: *mut u8, size: usize, count: usize, file: *mut FILE) -> usize {
    unsafe {
        let file = &mut *file;

        let bytes = size * count;
        let remaining = file.data.len() - file.pos;
        let amount = bytes.min(remaining);

        core::ptr::copy_nonoverlapping(file.data.as_ptr().add(file.pos), ptr, amount);
        file.pos += amount;

        amount / size
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fwrite(
    ptr: *const u8,
    size: usize,
    count: usize,
    file: *mut FILE,
) -> usize {
    unsafe {
        let file = &mut *file;

        let bytes = size * count;
        let src = core::slice::from_raw_parts(ptr, bytes);

        if file.pos + bytes > file.data.len() {
            file.data.resize(file.pos + bytes, 0);
        }

        file.data[file.pos..file.pos + bytes].copy_from_slice(src);
        file.pos += bytes;

        count
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftell(file: *mut FILE) -> i64 {
    unsafe {
        let file = &mut *file;
        file.pos as i64
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fseek(file: *mut FILE, offset: i64, whence: i32) -> i32 {
    unsafe {
        if file.is_null() {
            return -1;
        }

        let file = &mut *file;

        match whence {
            0 => {
                file.pos = offset.max(0) as usize;
            }

            1 => {
                file.pos = (file.pos as i64 + offset).max(0) as usize;
            }

            2 => {
                file.pos = (file.data.len() as i64 + offset).max(0) as usize;
            }

            _ => return -1,
        }

        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fflush(_file: *mut FILE) -> i32 {
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fclose(file: *mut FILE) -> i32 {
    unsafe {
        if file.is_null() {
            return -1;
        }

        let file = Box::from_raw(file);

        crate::println!("fclose {}", file.name);

        if file.write {
            crate::lib::fs::write(&file.name, &file.data);
        }

        0
    }
}
