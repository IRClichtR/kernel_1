# Project Structure Overview

## 1. boot/boot.asm

- This is the assembly code that handles the multiboot header and initial boot process. It's responsible for setting up the initial environment before handing control to the Rust kernel.

## 2. build/rust_os.bin

- This is the compiled binary output of the OS.

## 3. Cargo.toml & Cargo.lock

- These are standard Rust package management files. Cargo.toml defines the project dependencies and configuration, while Cargo.lock tracks the exact versions of dependencies.

## 4. i686-unknown-none.json

- This is a target specification file that tells the Rust compiler how to build for a bare-metal i686 architecture with no underlying OS.

## 5. linker.ld

- This is a linker script that tells the linker how to arrange the sections of the compiled code in memory. It's crucial for OS development because it specifies where in memory the kernel will be loaded.

## 6. src/main.rs

- The entry point of the Rust kernel code.

## 7. Makefile

- Contains the build instructions to compile the assembly, Rust code, and link everything together.