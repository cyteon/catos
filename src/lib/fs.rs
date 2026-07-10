use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use spin::mutex::Mutex;

use crate::drivers::console::{GREEN, RESET};

pub struct RamFile {
    pub name: String,
    pub data: Vec<u8>,
}

pub static FS: Mutex<Vec<RamFile>> = Mutex::new(Vec::new());

pub fn init(initrd: &'static [u8]) {
    super::initrd::init(initrd);
    crate::println!("[ {}OK{} ] initrd initialized", GREEN, RESET);

    let mut fs = FS.lock();
    for file in super::initrd::files(initrd) {
        fs.push(RamFile {
            name: file.name.to_string(),
            data: file.data.to_vec(),
        });
    }
}

pub fn list() -> Vec<(u32, String)> {
    let fs = FS.lock();

    fs.iter()
        .map(|file| (file.data.len() as u32, file.name.clone()))
        .collect()
}

pub fn read(name: &str) -> Option<Vec<u8>> {
    FS.lock()
        .iter()
        .find(|file| file.name == name)
        .map(|file| file.data.clone())
}

pub fn write(name: &str, data: &[u8]) {
    let mut fs = FS.lock();

    if let Some(file) = fs.iter_mut().find(|file| file.name == name) {
        file.data = data.to_vec();
    } else {
        fs.push(RamFile {
            name: name.to_string(),
            data: data.to_vec(),
        });
    }
}

pub fn remove(name: &str) {
    let mut fs = FS.lock();

    if let Some(pos) = fs.iter().position(|file| file.name == name) {
        fs.remove(pos);
    }
}
