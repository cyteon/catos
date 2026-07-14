use core::sync::atomic::Ordering;

use alloc::{string::String, vec::Vec};
use spin::mutex::Mutex;

use crate::{
    TICKS,
    drivers::{
        self,
        console::{RED, RESET},
        pit::TICK_HZ,
    },
    lib::{
        fs, initrd,
        keys::{Key, pop_key},
        tasks,
    },
    print, println,
};

static HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn shell_loop() -> ! {
    let mut last_blink = 0;
    let mut history_index = 0;

    let mut line = String::new();
    print!("catos> ");

    loop {
        x86_64::instructions::hlt();

        let ticks = TICKS.load(Ordering::Relaxed);
        if ticks / ((drivers::pit::TICK_HZ as u64) / 2) != last_blink {
            last_blink = ticks / ((drivers::pit::TICK_HZ as u64) / 2);
            drivers::console::tick_cursor();
        }

        while let Some(key) = pop_key() {
            match key {
                Key::Char(c) => match c {
                    '\n' => {
                        println!();
                        run_command(&line);

                        let mut history = HISTORY.lock();
                        history.push(line.clone());

                        history_index = history.len();

                        line.clear();
                        print!("catos> ");
                    }

                    '\x08' => {
                        if line.pop().is_some() {
                            print!("\x08 \x08");
                        }
                    }

                    '\x03' => {
                        line.clear();
                        println!("^C");
                        print!("catos> ");
                    }

                    c => {
                        line.push(c);
                        print!("{}", c);
                    }
                },

                Key::Up => {
                    if history_index > 0 {
                        history_index -= 1;
                    }

                    let history = HISTORY.lock();

                    if history_index < history.len() {
                        for _ in 0..line.len() {
                            print!("\x08 \x08");
                        }

                        line = history[history_index].clone();
                        print!("{}", line);
                    }
                }

                Key::Down => {
                    let history = HISTORY.lock();

                    if history.len() > 0 && history_index < history.len() - 1 {
                        history_index += 1;
                    } else {
                        history_index = history.len();
                    }

                    for _ in 0..line.len() {
                        print!("\x08 \x08");
                    }

                    if history_index < history.len() {
                        line = history[history_index].clone();
                        print!("{}", line);
                    } else {
                        line.clear();
                    }
                }

                _ => {}
            }
        }

        tasks::block_task(0);
    }
}

pub fn run_command(line: &str) {
    let mut args = line.split_whitespace();
    let Some(command) = args.next() else { return };
    let rest = args.clone().collect::<Vec<_>>().join(" ");

    match command {
        "help" => {
            println!("commands:");
            println!("  doom                - launch doom");
            println!("  help                - show this help message");
            println!("  echo <text>         - print <text> to the console");
            println!("  uptime              - show how long the system has been running");
            println!("  panic <msg>         - panic with <msg>");
            println!("  int3                - trigger a breakpoint exception");
            println!("  initrd              - show initrd debug info");
            println!("  ls                  - list files in the initrd");
            println!("  cat <file>          - print the contents of <file> in the initrd");
            println!("  write <file> <text> - write <text> to <file> in the initrd");
            println!("  rm <file>           - remove <file> from the initrd");
            println!("  ps                  - list running tasks");
            println!("  sleeptest           - sleeps for 5 seconds then prints");
            println!("  clear               - clear the console");
            println!("  fbtest              - launch a tiny test game, controls are w/a/s/d/esc");
        }

        "doom" => {
            println!("launching doom...");
            tasks::spawn_task("doom", crate::doom::run);
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

        "ps" => tasks::with_tasks(|tasks| {
            for task in tasks.iter() {
                println!(
                    "id={} name={} state={:?} rsp={:#x} stack={:#x}..{:#x}",
                    task.id, task.name, task.state, task.rsp, task.stack_bottom, task.stack_top
                );
            }
        }),

        "sleeptest" => {
            tasks::spawn_task("sleeptest", sleep_test);
        }

        "clear" => {
            drivers::console::clear();
        }

        "fbtest" => {
            tasks::spawn_task("fbtest", crate::lib::fbtest::fbtest);
        }

        _ => println!("{}unknown command: {}{}", RED, command, RESET),
    }
}

extern "C" fn sleep_test() {
    println!("task sleeping");
    tasks::sleep(5 * 1000);
    println!("task woke up");
}
