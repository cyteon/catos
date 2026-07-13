use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use spin::mutex::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

use crate::lib::memory::{self, STACK_BOTTOM, STACK_TOP};

pub struct Task {
    pub id: u64,
    pub name: String,
    pub rsp: u64,
    pub stack_bottom: u64,
    pub stack_top: u64,
}

pub static TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());

pub static CURRENT: AtomicUsize = AtomicUsize::new(0);

const TASK_STACK_REGION: u64 = 0xffff_b000_0000_0000;
const TASK_SLOT_SIZE: u64 = 64 * 4096;
const TASK_STACK_PAGES: u64 = 16;

pub fn init() {
    TASKS.lock().push(Task {
        id: 0,
        name: "shell".to_string(),
        rsp: 0,
        stack_bottom: STACK_BOTTOM,
        stack_top: STACK_TOP,
    });
}

fn slot_guard(id: u64) -> u64 {
    TASK_STACK_REGION + id * TASK_SLOT_SIZE
}

fn prepare_stack(top: u64, entry: extern "C" fn() -> !) -> u64 {
    unsafe {
        let mut stack = top as *mut u64;
        stack = stack.offset(-1);
        *stack = entry as u64;

        for _ in 0..6 {
            stack = stack.offset(-1);
            *stack = 0;
        }

        stack as u64
    }
}

pub fn spawn_task(name: &str, entry: extern "C" fn() -> !) -> u64 {
    let mut tasks = TASKS.lock();

    let id = tasks.len() as u64;
    let stack_bottom = slot_guard(id) + 4096;
    memory::map_stack(stack_bottom, TASK_STACK_PAGES);

    let top = stack_bottom + TASK_STACK_PAGES * 4096;
    let rsp = prepare_stack(top, entry);

    tasks.push(Task {
        id,
        name: name.to_string(),
        rsp,
        stack_bottom,
        stack_top: top,
    });

    id
}

#[unsafe(naked)]
pub extern "C" fn switch(old_rsp: &mut u64, new_rsp: u64) {
    core::arch::naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp",
        "mov rsp, rsi",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
    );
}

pub fn with_tasks<R>(f: impl FnOnce(&mut Vec<Task>) -> R) -> R {
    without_interrupts(|| f(&mut TASKS.lock()))
}

pub fn schedule() {
    let mut tasks = TASKS.lock();
    if tasks.len() < 2 {
        return;
    }

    let current = CURRENT.load(Ordering::Relaxed);
    let next = (current + 1) % tasks.len();

    let old_rsp: *mut u64 = &mut tasks[current].rsp;
    let new_rsp = tasks[next].rsp;

    drop(tasks);

    CURRENT.store(next, Ordering::Relaxed);

    unsafe {
        switch(&mut *old_rsp, new_rsp);
    }
}
