use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::drivers::pic;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt[32].set_handler_fn(timer_handler);
        idt[33].set_handler_fn(keyboard_handler);

        idt
    };
}

pub fn init() {
    IDT.load()
}

extern "x86-interrupt" fn timer_handler(_frame: InterruptStackFrame) {
    pic::end_of_interrupt(0);
}

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    pic::end_of_interrupt(1);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::println!("\n--- EXCEPTION: BREAKPOINT ---");
    crate::println!("{:#?}\n", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    crate::println!(
        "\n--- EXCEPTION: DOUBLE FAULT (error code: {}) ---",
        error_code
    );
    crate::println!("{:#?}\n", stack_frame);

    crate::hlt()
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    crate::println!("\n--- EXCEPTION: PAGE FAULT ---");
    crate::println!("Accessed Address: {:?}", Cr2::read());
    crate::println!("Error Code: {:?}", error_code);
    crate::println!("{:#?}\n", stack_frame);

    crate::hlt()
}
