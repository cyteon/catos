use core::sync::atomic::Ordering;

use alloc::vec::Vec;

use crate::{
    TICKS,
    drivers::{
        console::{RED, RESET},
        pit::TICK_HZ,
    },
    lib::{fs, initrd},
    print, println,
};

pub fn run_command(line: &str) {
    let mut args = line.split_whitespace();
    let Some(command) = args.next() else { return };
    let rest = args.clone().collect::<Vec<_>>().join(" ");

    match command {
        "help" => {
            println!("commands:");
            println!("  help                - show this help message");
            println!("  echo <text>         - print <text> to the console");
            println!("  uptime              - show how long the system has been running");
            println!("  panic <msg>         - panic with <msg>");
            println!("  int3                - trigger a breakpoint exception");
            println!("  initrd              - show initrd debug info");
            println!("  ls                  - list files in the initrd");
            println!("  cat <file>          - print the contents of <file> in the initrd");
            println!("  write <file> <text> - write <text> to <file> in the initrd");
        }

        "echo" => {
            println!("{}", rest);
        }

        "uptime" => {
            let ticks = TICKS.load(Ordering::Relaxed);
            let seconds = ticks / TICK_HZ as u64;
            let minutes = seconds / 60;
            let hours = minutes / 60;

            println!(
                "up for {}h {}m {}s",
                hours as u64,
                minutes as u64 % 60,
                seconds as u64 % 60
            );
        }

        "panic" => {
            panic!("{}", rest);
        }

        "int3" => {
            x86_64::instructions::interrupts::int3();
        }

        "initrd" => match initrd::get() {
            None => println!("{}no initrd found{}", RED, RESET),

            Some(tar) => {
                println!("initrd found: {} bytes at {:p}", tar.len(), tar.as_ptr());

                let n = tar.len().min(64);

                for chunk in tar[..n].chunks(16) {
                    for byte in chunk {
                        print!("{:02x} ", byte);
                    }

                    println!();
                }
            }
        },

        "ls" => {
            for file in fs::list() {
                crate::println!("{:>8} {}", file.0, file.1);
            }
        }

        "cat" => {
            if let Some(file) = fs::read(&rest) {
                let content = core::str::from_utf8(&file).unwrap_or("<invalid utf-8>");
                println!("{}", content);
            } else {
                println!("{}file not found: {}{}", RED, rest, RESET);
            }
        }

        "write" => {
            if args.clone().count() < 2 {
                println!("{}usage: write <file> <data>{}", RED, RESET);
                return;
            }

            let filename = args.next().unwrap();
            let data = args.collect::<Vec<_>>().join(" ").into_bytes();

            fs::write(filename, &data);
        }

        "rm" => {
            if let Some(_) = fs::read(&rest) {
                fs::remove(&rest);
                println!("file removed: {}", rest);
            } else {
                println!("{}file not found: {}{}", RED, rest, RESET);
            }
        }

        _ => println!("{}unknown command: {}{}", RED, command, RESET),
    }
}
