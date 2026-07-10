use core::sync::atomic::Ordering;

use alloc::vec::Vec;

use crate::{
    TICKS,
    drivers::{
        console::{RED, RESET},
        pit::TICK_HZ,
    },
    lib::initrd::{self, INITRD},
    print, println,
};

pub fn run_command(line: &str) {
    let mut args = line.split_whitespace();
    let Some(command) = args.next() else { return };
    let rest = args.collect::<Vec<_>>().join(" ");

    match command {
        "help" => {
            println!("commands:");
            println!("  help        - show this help message");
            println!("  echo <text> - print <text> to the console");
            println!("  uptime      - show how long the system has been running");
            println!("  panic <msg> - panic with <msg>");
            println!("  int3        - trigger a breakpoint exception");
            println!("  initrd      - show initrd debug info");
            println!("  ls          - list files in the initrd");
            println!("  cat <file>  - print the contents of <file> in the initrd");
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
            if let Some(tar) = initrd::get() {
                for file in initrd::files(tar) {
                    crate::println!("{:>8} {}", file.data.len(), file.name);
                }
            } else {
                crate::println!("{}no initrd found{}", RED, RESET);
            }
        }

        "cat" => {
            if let Some(tar) = initrd::get() {
                match initrd::find(tar, &rest) {
                    Some(file) => match core::str::from_utf8(file.data) {
                        Ok(text) => crate::println!("{}", text),
                        Err(_) => crate::println!("{}file is not valid UTF-8{}", RED, RESET),
                    },
                    None => crate::println!("{}file not found: {}{}", RED, rest, RESET),
                }
            } else {
                crate::println!("{}no initrd found{}", RED, RESET);
            }
        }

        _ => println!("{}unknown command: {}{}", RED, command, RESET),
    }
}
