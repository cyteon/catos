use lazy_static::lazy_static;
use x86_64::{
    VirtAddr,
    instructions::tables::load_tss,
    registers::segmentation::{CS, DS, ES, SS, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
struct Stack([u8; STACK_SIZE]);

static mut DOUBLE_FAULT_STACK: Stack = Stack([0; STACK_SIZE]);

pub fn init() {
    GDT.0.load();

    unsafe {
        CS::set_reg(GDT.1.code);
        SS::set_reg(GDT.1.data);
        DS::set_reg(GDT.1.data);
        ES::set_reg(GDT.1.data);
        load_tss(GDT.1.tss);
    }
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let start = VirtAddr::from_ptr(&raw const DOUBLE_FAULT_STACK);
            start + STACK_SIZE as u64
        };

        tss
    };
}

struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
    tss: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code = gdt.append(Descriptor::kernel_code_segment());
        let data = gdt.append(Descriptor::kernel_data_segment());
        let tss = gdt.append(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { code, data, tss })
    };
}
