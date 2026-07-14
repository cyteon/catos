pub mod c;

use core::ffi::{c_char, c_int};
use core::sync::atomic::Ordering;

use alloc::vec::Vec;
use alloc::{ffi::CString, vec};
use pc_keyboard::KeyCode;

use crate::drivers::fb::FbInfo;
use crate::drivers::pit::TICK_HZ;
use crate::lib::rawkeys::{RawKey, pop_raw};
use crate::lib::tasks;

unsafe extern "C" {
    fn doomgeneric_Create(argc: c_int, argv: *mut *mut c_char);
    fn doomgeneric_Tick();

    static mut DG_ScreenBuffer: *mut u32;
}

static mut FRAMEBUFFER: Option<&FbInfo> = None;

#[unsafe(no_mangle)]
pub extern "C" fn DG_Init() {
    crate::println!("DG_Init called");
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_DrawFrame() {
    unsafe {
        let Some(fb) = FRAMEBUFFER else {
            crate::println!("FRAMEBUFFER is null");
            return;
        };

        if DG_ScreenBuffer.is_null() {
            crate::println!("DG_ScreenBuffer is null");
            return;
        }

        let doom = core::slice::from_raw_parts(DG_ScreenBuffer, 640 * 400);

        if fb.width == 640 && fb.height == 400 {
            for y in 0..400 {
                let src = DG_ScreenBuffer.add(y * 640);
                let dst = fb.addr.add(y * fb.pitch).cast::<u32>();
                core::ptr::copy_nonoverlapping(src, dst, 640);
            }
        } else {
            for y in 0..fb.height {
                for x in 0..fb.width {
                    let sx = x * 640 / fb.width;
                    let sy = y * 400 / fb.height;

                    let color = doom[sy * 640 + sx];

                    let offset = y * fb.pitch + x * (fb.bpp / 8);
                    fb.addr.add(offset).cast::<u32>().write_volatile(color);
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_SleepMs(ms: u32) {
    tasks::sleep(ms as u64);
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_GetTicksMs() -> u32 {
    crate::TICKS.load(Ordering::Relaxed) as u32 * (1000 / TICK_HZ)
}

fn doom_key(code: KeyCode) -> Option<u8> {
    Some(match code {
        KeyCode::W | KeyCode::ArrowUp => 0xAD,
        KeyCode::S | KeyCode::ArrowDown => 0xAF,
        KeyCode::A | KeyCode::ArrowLeft => 0xAC,
        KeyCode::D | KeyCode::ArrowRight => 0xAE,
        KeyCode::LControl | KeyCode::RControl => 0xA3,
        KeyCode::Spacebar => 0xA2,
        KeyCode::LShift | KeyCode::RShift => 0x80 + 0x36,
        KeyCode::Escape => 27,
        KeyCode::Return => 13,
        KeyCode::Tab => 9,
        KeyCode::Key1 => b'1',
        KeyCode::Key2 => b'2',
        KeyCode::Key3 => b'3',
        KeyCode::Key4 => b'4',
        KeyCode::Key5 => b'5',
        KeyCode::Key6 => b'6',
        KeyCode::Key7 => b'7',
        KeyCode::Y => b'y',
        KeyCode::N => b'n',
        _ => return None,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_GetKey(pressed: *mut c_int, key: *mut u8) -> c_int {
    loop {
        let Some(event) = pop_raw() else {
            return 0;
        };

        let (code, down) = match event {
            RawKey::Press(code) => (code, 1),
            RawKey::Release(code) => (code, 0),
        };

        if let Some(k) = doom_key(code) {
            unsafe {
                *pressed = down;
                *key = k;
            }

            return 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_SetWindowTitle(title: *const c_char) {}

pub extern "C" fn run() {
    x86_64::instructions::interrupts::enable();

    let task_id = tasks::CURRENT.load(Ordering::Relaxed);

    let Some(fb) = crate::drivers::fb::acquire(task_id as u64) else {
        crate::println!("failed to acquire framebuffer for doom");
        return;
    };

    for i in 0..fb.height {
        for j in 0..fb.width {
            unsafe {
                let offset = i * fb.pitch + j * (fb.bpp / 8);
                fb.addr.add(offset).cast::<u32>().write_volatile(0x000000);
            }
        }
    }

    let args = vec![
        CString::new("doom").unwrap().into_raw(),
        CString::new("-iwad").unwrap().into_raw(),
        CString::new("doom1.wad").unwrap().into_raw(),
    ];

    let mut argv: Vec<*mut c_char> = args.into_iter().map(|s| s as *mut c_char).collect();

    unsafe {
        FRAMEBUFFER = Some(fb);

        doomgeneric_Create(argv.len() as c_int, argv.as_mut_ptr());

        loop {
            doomgeneric_Tick();
            tasks::sleep(1);
        }
    }
}
