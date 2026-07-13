use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use spin::mutex::Mutex;

use crate::lib::memory::{STACK_BOTTOM, STACK_TOP};

pub struct Task {
    pub id: u64,
    pub name: String,
    pub rsp: u64,
    pub stack_bottom: u64,
    pub stack_top: u64,
}

pub static TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());

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
