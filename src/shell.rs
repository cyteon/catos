use core::sync::atomic::Ordering;

use alloc::vec::Vec;

use crate::{
    TICKS,
    drivers::console::{RED, RESET},
    println,
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
        }

        "echo" => {
            println!("{}", rest);
        }

        "uptime" => {
            let ticks = TICKS.load(Ordering::Relaxed);
            let seconds = ticks as f64 / 18.2065;
            let minutes = seconds / 60.0;
            let hours = minutes / 60.0;

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

        _ => println!("{}unknown command: {}{}", RED, command, RESET),
    }
}
