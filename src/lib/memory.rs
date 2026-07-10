use limine::memory_map::{Entry, EntryType};
use linked_list_allocator::LockedHeap;
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
pub const HEAP_SIZE: u64 = 1024 * 1024;

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

pub fn init(hhdm_offset: u64, entries: &'static [&'static Entry]) {
    let mut mapper = unsafe { page_table(hhdm_offset) };
    let mut frame_allocator = BootFrameAllocator::new(entries);

    let start = Page::<Size4KiB>::containing_address(VirtAddr::new(HEAP_START));
    let end = Page::<Size4KiB>::containing_address(VirtAddr::new(HEAP_START + HEAP_SIZE - 1));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

    for page in Page::range_inclusive(start, end) {
        let frame = frame_allocator.allocate_frame().expect("no more frames");

        unsafe {
            mapper
                .map_to(page, frame, flags, &mut frame_allocator)
                .expect("map failed")
                .flush();
        }
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as *mut u8, HEAP_SIZE as usize);
    }
}
