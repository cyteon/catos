use core::sync::atomic::{AtomicU64, Ordering};

use limine::memory_map::{Entry, EntryType};
use linked_list_allocator::LockedHeap;
use spin::mutex::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: u64 = 0xffff_9000_0000_0000;
pub const HEAP_SIZE: u64 = 32 * 1024 * 1024;
pub const STACK_GUARD: u64 = 0xffff_a000_0000_0000;
pub const STACK_BOTTOM: u64 = STACK_GUARD + 4096;
pub const STACK_PAGES: u64 = 16;
pub const STACK_TOP: u64 = STACK_BOTTOM + STACK_PAGES * 4096;

pub static FRAME_ALLOCATOR: Mutex<Option<BootFrameAllocator>> = Mutex::new(None);
pub static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0);

pub struct BootFrameAllocator {
    entries: &'static [&'static Entry],
    next: usize,
}

impl BootFrameAllocator {
    pub fn new(entries: &'static [&'static Entry]) -> Self {
        Self { entries, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.entries
            .iter()
            .filter(|e| e.entry_type == EntryType::USABLE)
            .map(|e| e.base..(e.base + e.length))
            .flat_map(|r| r.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);

        if frame.is_some() {
            self.next += 1;
        }

        frame
    }
}

pub unsafe fn page_table(hhdm_offset: u64) -> OffsetPageTable<'static> {
    let (frame, _) = Cr3::read();
    let virt = VirtAddr::new(hhdm_offset + frame.start_address().as_u64());
    let table: &mut PageTable = unsafe { &mut *virt.as_mut_ptr() };

    unsafe { OffsetPageTable::new(table, VirtAddr::new(hhdm_offset)) }
}

fn map_range(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut BootFrameAllocator,
    start: u64,
    size: u64,
) {
    let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start));
    let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start + size - 1));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator.allocate_frame().expect("no more frames");

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("map failed")
                .flush();
        }
    }
}

pub fn init(hhdm_offset: u64, entries: &'static [&'static Entry]) {
    HHDM_OFFSET.store(hhdm_offset, Ordering::Relaxed);

    let mut mapper = unsafe { page_table(hhdm_offset) };
    let mut frame_allocator = BootFrameAllocator::new(entries);

    map_range(&mut mapper, &mut frame_allocator, HEAP_START, HEAP_SIZE);
    map_range(
        &mut mapper,
        &mut frame_allocator,
        STACK_BOTTOM,
        STACK_PAGES * 4096,
    );

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as *mut u8, HEAP_SIZE as usize);
    }

    *FRAME_ALLOCATOR.lock() = Some(frame_allocator);
}

pub fn map_stack(bottom: u64, pages: u64) {
    let mut mapper = unsafe { page_table(HHDM_OFFSET.load(Ordering::Relaxed)) };
    let mut frame_allocator = FRAME_ALLOCATOR.lock();

    let frame_allocator = frame_allocator
        .as_mut()
        .expect("frame allocator not initialized");

    map_range(&mut mapper, frame_allocator, bottom, pages * 4096);
}
