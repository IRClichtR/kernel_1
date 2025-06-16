# rust_kernel
rust minimal kernel

# PART0 -- Setup the environment

We need a binary that doesn't rely on the current operating system, but a freestanding binary. That means that:
* We should use the 2018 version of rust since since it offers compatibility with low level enviromnment and improves the allocatiion process. The 2021 version does not provide any particular advantage in these aspects.
* We cannot use std lib because it relies on the operating system for features like threads, files or networking or libc.
* For the same reason, we cannot use the main() function, as a typical Rust binary starts with crt0, which sets up a C environment, invokes the Rust runtime, and ultimately calls the main() function.

# PART1 -- Setup the bootloader

## 1- Definition
A bootloader is the initial program that executes when a computing device powers on. Its primary function is to locate and load the operating system into RAM from storage media. The bootloader searches the device's firmware for information regarding the location of the operating system components, initializes essential hardware, and prepares the environment necessary for the operating system to start functioning. Once the bootloader completes its tasks, it transfers control to the operating system kernel.

For our project, the bootloader should follow Multiboot protocol that loads into 32-bit protected mode without dividing memory space into blocks (paging).

## 2- Build
When the assembly code is calling kernel_main ```call kernel_main```, it's looking for a symbol named __exactly__ `kernel_main` in the binary.
The synbol must be:
- defined in the rust code, 
- not name-magled (Rust mangles function names for type safety)
- using the C calling convention (to be readable by the assembly code)
- Exported as a global symbol during linking

```rust
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {}
```

Since the kernel will be in an external source, do not forget to declare it as external into you asm code befor you call it
```asm
extern kernel_main
[...]
call kernel_main
```
`Makefile` command to compile the bootlader into a usable binary
```bash
nasm -f elf32 $(BOOT_DIR)/boot.asm -o $(BUILD_DIR)/boot.bin
```

# PART2 -- The kernel file

## Cargo compiler specific instructions

## The linker