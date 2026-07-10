# CatOS

A small AMD64 kernel written in Rust. It uses limine as a bootloader and is a 64-bit OS. \
It has a shell with basic commands, and a read+write ram filesystem which is initialized from initrd.

For testing qemu is primarily used, and "cargo run" will set up limine stuff, make the iso and run it in qemu. \
But the ISO has been tested on real hardware and does boot.

<img width="554" height="561" alt="image" src="https://github.com/user-attachments/assets/2f9f98de-bc46-4a69-ab66-2b6239f1875f" />

## Building

### From releases
You can find a prebuilt iso image in the [releases](https://github.com/cyteon/catos/releases).

### From source
1. Install xorriso and qemu-system-x86_64

2. Install the rustup bare betal target
```bash
rustup target add x86_64-unknown-none
```

3. Install the nightly toolchain
```bash
rustup install nightly
```

4. Run the project locally with qemu (only tested for linux)
```bash
cargo run
```

5. If you wanna run it in another vm or on real hardware, after running cargo run you can get the iso from `target/catos.iso`
