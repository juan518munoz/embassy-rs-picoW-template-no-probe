# Embassy rs - Pico W template - no probe
Simple template to compile and flash to a Pico W or any other rp2040 device. No probe required.
Example taken from [embassy-rs rp examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp/src/bin), use it to change `main.rs` to your needs.

## Setup & Run
- Install elf2uf2-rs
```bash
cargo install elf2uf2-rs
```
- Run cargo
```bash
cargo run --release
```

Pico W led should start blinking.