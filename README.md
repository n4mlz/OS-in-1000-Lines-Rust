## Setup

```sh
$ sudo apt install qemu-system-riscv32
$ rustup install nightly
$ rustup target add riscv32i-unknown-none-elf
$ ./setup.sh  # download OpenSBI
$ cargo install cargo-binutils  # optional
$ cargo run
```
