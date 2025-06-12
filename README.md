## Usage

**Setup**

```sh
$ sudo apt install qemu-system-riscv32
$ rustup install nightly
$ rustup target add riscv32i-unknown-none-elf
$ ./setup.sh  # download OpenSBI
$ cargo install cargo-binutils  # optional
$ rustup component add llvm-tools-preview  # optional
```

**Build and Run**

```sh
$ cargo run
```

**Debugging**

```sh
$ cargo objdump --bin kernel -- --source
```
