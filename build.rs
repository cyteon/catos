use std::{fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=doom");

    let doom_sources: Vec<PathBuf> = fs::read_dir("doom/src")
        .unwrap()
        .chain(fs::read_dir("doom/shim").unwrap())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "c"))
        .collect();

    let gcc_include = Command::new("gcc")
        .arg("-print-file-name=include")
        .output()
        .unwrap();

    let gcc_include = String::from_utf8(gcc_include.stdout)
        .unwrap()
        .trim()
        .to_string();

    cc::Build::new()
        .files(&doom_sources)
        .include("doom/shim")
        .include("doom/src")
        .flag("-ffreestanding")
        .flag("-fno-stack-protector")
        .flag("-fno-builtin")
        .flag("-mno-red-zone")
        .flag("-mcmodel=kernel")
        .flag("-fno-pic")
        .flag("-nostdinc")
        .flag("-idirafter")
        .flag(&gcc_include)
        .flag("-w")
        .compile("doom");
}
