# CatOS

A small AMD64 kernel written in Rust.

<img width="554" height="561" alt="image" src="https://github.com/user-attachments/assets/2f9f98de-bc46-4a69-ab66-2b6239f1875f" />


## Building

### From releases
You can find a prebuilt iso image in the [releases](https://github.com/cyteon/catos/releases).

### From source
1. Install the rustup bare betal target
```bash
rustup target add x86_64-unknown-none
```

2. Install the nightly toolchain
```bash
rustup install nightly
```

3. Run the project locally with qemu (only tested for linux)
```bash
cargo run
```

4. If you wanna run it in another vm or on real hardware, after running cargo run you can get the iso from `target/catos.iso`
