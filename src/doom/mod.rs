pub mod c;

use core::ffi::{c_char, c_int};

use alloc::vec::Vec;
use alloc::{ffi::CString, vec};

use crate::lib::tasks;

unsafe extern "C" {
    fn doomgeneric_Create(argc: c_int, argv: *mut *mut c_char);
    fn doomgeneric_Tick();
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_Init() {}

#[unsafe(no_mangle)]
pub extern "C" fn DG_DrawFrame() {}

#[unsafe(no_mangle)]
pub extern "C" fn DG_SleepMs(ms: u32) {
    tasks::sleep(ms as u64);
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_GetTicksMs() -> u32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_GetKey(pressed: *mut c_int, key: *mut u8) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_SetWindowTitle(title: *const c_char) {}

pub extern "C" fn run() {
    let args = vec![
        CString::new("doom").unwrap().into_raw(),
        CString::new("-iwad").unwrap().into_raw(),
        CString::new("doom1.wad").unwrap().into_raw(),
    ];

    let mut argv: Vec<*mut c_char> = args.into_iter().map(|s| s as *mut c_char).collect();

    unsafe {
        doomgeneric_Create(argv.len() as c_int, argv.as_mut_ptr());

        loop {
            doomgeneric_Tick();
            tasks::sleep(1);
        }
    }
}
