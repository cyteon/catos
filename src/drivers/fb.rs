use core::sync::atomic::{AtomicUsize, Ordering};
use limine::framebuffer::Framebuffer;
use spin::Once;

pub struct FbInfo {
    pub addr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bpp: usize,
}

static INFO: Once<FbInfo> = Once::new();
pub static OWNER: AtomicUsize = AtomicUsize::new(usize::MAX); // 0 = console, n = task id, max = unowned

unsafe impl Send for FbInfo {}
unsafe impl Sync for FbInfo {}

pub fn init(fb: Framebuffer) {
    INFO.call_once(|| FbInfo {
        addr: fb.addr(),
        width: fb.width() as usize,
        height: fb.height() as usize,
        pitch: fb.pitch() as usize,
        bpp: fb.bpp() as usize,
    });

    OWNER.store(0, Ordering::Release);
}

pub fn info() -> &'static FbInfo {
    INFO.get().expect("framebuffer not initialized")
}

pub fn acquire(task_id: u64) -> Option<&'static FbInfo> {
    if OWNER
        .compare_exchange(0, task_id as usize, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        Some(INFO.get().expect("framebuffer not initialized"))
    } else {
        None
    }
}

pub fn release_if_owner(task_id: u64) {
    let _ = OWNER.compare_exchange(task_id as usize, 0, Ordering::Release, Ordering::Relaxed);
}

pub fn force_console() {
    OWNER.store(0, Ordering::Release);
}
