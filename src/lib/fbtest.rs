// test fb locking and behaviour, for doom porting

use core::sync::atomic::Ordering;

use pc_keyboard::KeyCode;

use crate::{
    drivers::{console, fb},
    lib::{
        rawkeys::{self, RawKey},
        tasks::{self, CURRENT},
    },
};

pub extern "C" fn fbtest() {
    x86_64::instructions::interrupts::enable();

    let task_id = CURRENT.load(Ordering::Relaxed);

    let Some(fb) = fb::acquire(task_id as u64) else {
        crate::println!("fbtest failed to acquire framebuffer");
        return;
    };

    for y in 0..fb.height {
        for x in 0..fb.width {
            let pixel_offset = y * fb.pitch + x * (fb.bpp / 8);
            unsafe {
                fb.addr
                    .add(pixel_offset)
                    .cast::<u32>()
                    .write_volatile(0xFFFFFF);
            }
        }
    }

    let mut x: i32 = fb.width as i32 / 2 - 16;
    let mut y: i32 = fb.height as i32 / 2 - 16;

    let mut up = false;
    let mut down = false;
    let mut left = false;
    let mut right = false;

    loop {
        while let Some(k) = rawkeys::pop_raw() {
            let (code, pressed) = match k {
                RawKey::Press(code) => (code, true),
                RawKey::Release(code) => (code, false),
            };

            match code {
                KeyCode::W => up = pressed,
                KeyCode::S => down = pressed,
                KeyCode::A => left = pressed,
                KeyCode::D => right = pressed,
                KeyCode::Escape => {
                    for y in 0..fb.height {
                        for x in 0..fb.width {
                            let pixel_offset = y * fb.pitch + x * (fb.bpp / 8);
                            unsafe {
                                fb.addr
                                    .add(pixel_offset)
                                    .cast::<u32>()
                                    .write_volatile(0x000000);
                            }
                        }
                    }

                    fb::release_if_owner(task_id as u64);

                    console::clear();

                    return;
                }
                _ => {}
            }
        }

        for dy in 0..32 {
            for dx in 0..32 {
                let pixel_offset = (dy + y as usize) * fb.pitch + (dx + x as usize) * (fb.bpp / 8);

                unsafe {
                    fb.addr
                        .add(pixel_offset)
                        .cast::<u32>()
                        .write_volatile(0xFFFFFF);
                }
            }
        }

        if up {
            y -= 1;
        }

        if down {
            y += 1;
        }

        if left {
            x -= 1;
        }

        if right {
            x += 1;
        }

        x = x.clamp(0, fb.width as i32 - 16);
        y = y.clamp(0, fb.height as i32 - 16);

        for dy in 0..32 {
            for dx in 0..32 {
                let pixel_offset = (dy + y as usize) * fb.pitch + (dx + x as usize) * (fb.bpp / 8);

                unsafe {
                    fb.addr
                        .add(pixel_offset)
                        .cast::<u32>()
                        .write_volatile(0x00FF00);
                }
            }
        }

        tasks::sleep(10);
    }
}
