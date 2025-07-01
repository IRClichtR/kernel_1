# Rust Kernel Project

The purpose of this project is to setup a minimal viable kernel in Rust. The project does not have production ambitions. Since the best way to learn things is to do some project, we decided we would learn about Rust system programming and kernel level architecture by recreating a kernel from scratch. The following README is an attempt at describing our journey so far.

## Usage

This project relies on Docker. You need to install it before any building → https://docs.docker.com/get-started/get-docker/

Use the Makefile to make the project.

## PART 0 -- Setup the Environment

We need a binary that doesn't rely on the current operating system, but a freestanding binary. That means that:

* We should use the 2018 version of Rust since it offers compatibility with low level environments and improves the allocation process. The 2021 version does not provide any particular advantage in these aspects.
* We cannot use std lib because it relies on the operating system for features like threads, files, networking, or libc.
* For the same reason, we cannot use the main() function, as a typical Rust binary starts with crt0, which sets up a C environment, invokes the Rust runtime, and ultimately calls the main() function.

## PART 1 -- Setup the Bootloader

### 1 - Definition

A bootloader is the initial program that executes when a computing device powers on. Its primary function is to locate and load the operating system into RAM from storage media. The bootloader searches the device's firmware for information regarding the location of the operating system components, initializes essential hardware, and prepares the environment necessary for the operating system to start functioning. Once the bootloader completes its tasks, it transfers control to the operating system kernel.

For our project, the bootloader should follow the Multiboot protocol that loads into 32-bit protected mode without dividing memory space into blocks (paging).

### 2 - Build

When the assembly code is calling kernel_main `call kernel_main`, it's looking for a symbol named **exactly** `kernel_main` in the binary.

The symbol must be:
- Defined in the Rust code
- Not name-mangled (Rust mangles function names for type safety) 
- Using the C calling convention (to be readable by the assembly code)
- Exported as a global symbol during linking

```rust
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {}
```

Since the kernel will be in an external source, do not forget to declare it as external in your asm code before you call it:

```asm
extern kernel_main
[...]
call kernel_main
```

`Makefile` command to compile the bootloader into a usable binary:

```bash
nasm -f elf32 $(BOOT_DIR)/boot.asm -o $(BUILD_DIR)/boot.bin
```

## PART 2 -- The Kernel File

### Cargo Compiler Specific Instructions

The Rust compiler needs specific configuration to build a freestanding binary that can run without an operating system. This involves several key components:

**Target Specification**: We need to create a custom target that doesn't assume the presence of an operating system. This involves creating a `.json` target file that specifies the i386 architecture, data layout, and various compilation flags.

**Cargo Configuration**: The `Cargo.toml` file must be configured to disable the standard library and specify our custom target. Key configurations include:
- `#![no_std]` attribute to disable the standard library
- `#![no_main]` attribute to disable the standard main function
- Custom panic handler since we can't use the default one from std

**Build Profile**: Debug symbols and optimization levels need to be carefully configured for kernel development, balancing debuggability with performance and size constraints.

### The Linker

The linker script is crucial for kernel development as it defines how the final binary is laid out in memory. For our kernel, the linker script must:

**Memory Layout**: Define the memory sections where different parts of the kernel will be loaded. This typically includes sections for code (.text), read-only data (.rodata), initialized data (.data), and uninitialized data (.bss).

**Entry Point**: Specify the entry point of our kernel, which will be called by the bootloader. This needs to match the symbol we export from our Rust code.

**Alignment Requirements**: Ensure proper alignment of sections according to the i386 architecture requirements, particularly important for x86 systems where certain data structures must be aligned on specific boundaries.

**Multiboot Header**: If using the Multiboot specification, the linker script must ensure the Multiboot header is placed at the correct location within the first 8KB of the kernel binary so the bootloader can find it.

### Building the Kernel Image

The kernel image creation process involves several steps that transform our Rust source code into a bootable binary:

**Compilation**: The Rust compiler generates object files from our source code using the custom i386 target specification. This process must handle the absence of the standard library and runtime.

**Linking**: The linker combines our compiled Rust objects with the bootloader object file, using our custom linker script to create the final executable layout. This step resolves all symbol references and creates the memory layout our bootloader expects.

**Image Creation**: The final step involves creating a bootable image file that can be loaded by virtualization software or written to physical media. This typically involves creating an ISO file or disk image that contains our kernel binary in a format that can be booted.

```Makefile make all ```

It works!!

# PART 3 -- Basic display integration

Now we've got a functional Kernel, but nothing in it. We nee to have some basic tools to display chars on the screen, Display 

## Implement the printk!() Macro

### Overview

The `printk` macro serves as our kernel's primary logging and output mechanism, similar to `printf` in C or `println!` in standard Rust. Since we cannot use the standard library's printing macros in our freestanding kernel environment, we must implement our own formatting and output system.

### Architecture

The printk implementation consists of several key components working together:

**LogLevel Enum**: Defines different severity levels for kernel messages, from Emergency (0) to Debug (7), following the standard syslog severity levels. Each level has a corresponding prefix that gets prepended to log messages for easy identification and filtering.

**Logger Struct**: Acts as the core formatting engine that implements the `core::fmt::Write` trait. This allows us to leverage Rust's built-in formatting infrastructure while directing output to our custom screen management system instead of standard output.

**Screen Integration**: The logger integrates with our screen management system through a global screen manager. It writes to the currently active screen buffer and handles the low-level details of character output, cursor positioning, and screen updates.

### Implementation Details

**Write Trait Implementation**: The `Logger` implements `core::fmt::Write`, which requires a `write_str` method. This method:
- Acquires a lock on the global screen manager to ensure thread-safe access
- Identifies the currently active screen for output
- Creates a writer instance for character-level output
- Writes the log level prefix followed by the formatted message
- Handles screen buffer flushing and cursor updates

**Macro Variants**: The `printk!` macro supports two usage patterns:
- `printk!(LogLevel::Error, "format string", args...)` - with explicit log level
- `printk!("format string", args...)` - using default log level

**Memory Safety**: The implementation carefully manages shared state through mutex locks and handles the absence of heap allocation by working directly with screen buffers and stack-allocated structures.

### Usage Examples

```rust
// Basic usage with default log level
printk!("Kernel initialized successfully");

// With specific log level
printk!(LogLevel::Error, "Failed to initialize device: {}", device_id);

// Formatted output with multiple parameters
printk!(LogLevel::Info, "Memory available: {} KB, Used: {} KB", total, used);
```

This implementation provides a familiar interface for kernel debugging and logging while respecting the constraints of our freestanding environment.


## Read from keyboard
For keyboard input handling, we implemented a simple but effective approach using direct PS/2 port communication. While a complete kernel would typically use interrupt handlers through an Interrupt Descriptor Table (IDT) and Global Descriptor Table (GDT), we chose to use polling-based input reading for simplicity and to focus on the core keyboard functionality.

### Architecture

The keyboard implementation consists of several key components:

**Port Communication**: Direct hardware communication using the PS/2 controller ports:
- `0x60` - Keyboard data port for reading scancodes
- `0x64` - Keyboard status port for checking data availability

**Scancode Translation**: A comprehensive lookup table (`SCANCODE_TO_ASCII`) that maps hardware scancodes to ASCII characters, supporting the full range of standard keyboard keys including letters, numbers, symbols, and special keys.

**Event System**: A `KeyEvents` enum that abstracts keyboard input into meaningful events like character input, arrow keys, function keys, and custom actions like screen switching.

**State Management**: Global state tracking for modifier keys (Shift, Ctrl, Alt) and extended key sequences, allowing for complex key combinations and proper handling of multi-byte scancode sequences.

### Implementation Details

**Polling Mechanism**: Instead of interrupt-driven input, we use a polling approach where the kernel periodically checks if keyboard data is available. This is simpler to implement but requires active checking in the main kernel loop.

**Extended Key Handling**: Many keys (like arrow keys, function keys, and some special keys) send extended scancodes prefixed with `0xE0`. Our implementation properly handles these multi-byte sequences by maintaining state between calls.

**Modifier Key Support**: The system tracks the state of modifier keys (Shift, Ctrl, Alt) and applies appropriate transformations:
- Shift key converts lowercase to uppercase and transforms symbols (e.g., '1' becomes '!')
- Ctrl key enables special key combinations like Ctrl+Arrow for screen switching
- Key release events properly update modifier states

**Screen Integration**: Keyboard events are tightly integrated with our screen management system, allowing for:
- Direct cursor movement and character input
- Screen switching via Ctrl+Arrow key combinations
- Proper cursor positioning and screen buffer updates

### Key Features

**Character Input**: Standard alphanumeric and symbol input with proper Shift key handling for uppercase letters and alternate symbols.

**Navigation Keys**: Full support for arrow keys, Home, End, Page Up/Down with immediate cursor movement and screen updates.

**Editing Operations**: Backspace and Delete key support with proper cursor positioning and character erasure.

**Multi-Screen Support**: Custom key combinations (Ctrl+Left/Right arrows) for switching between virtual screens, demonstrating how custom kernel functionality can be bound to keyboard shortcuts.

**Robust State Management**: Proper handling of key press/release events ensures modifier keys work correctly and extended key sequences are processed without errors.


This approach provides responsive keyboard input while maintaining the simplicity needed for a learning-focused kernel implementation.

## PART 5 -- Screen Management and Virtual Terminals

### Overview

One of the key features of our kernel is the ability to manage multiple virtual screens (similar to virtual terminals in Linux). This allows users to switch between different isolated environments while maintaining separate command histories, cursor positions, and screen content. The implementation demonstrates advanced kernel concepts including thread-safe shared state management and hardware cursor control.

### Architecture

The screen management system consists of several interconnected components:

**Custom Spinlock Implementation**: Since we cannot use the standard library's synchronization primitives, we implemented our own `KSpinLock` using atomic operations. This provides thread-safe access to shared screen data without relying on operating system mutexes.

**Global Screen Manager**: A singleton pattern using `MaybeUninit` and unsafe code to create a globally accessible screen manager that can be safely initialized once and accessed throughout the kernel lifetime.

**Virtual Screen Abstraction**: Each virtual screen maintains its own independent buffer, cursor position, and state, allowing for complete isolation between different screen contexts.

**Physical Buffer Management**: The system manages the relationship between virtual screen buffers and the physical VGA text buffer at memory address `0xB8000`, ensuring only the active screen is displayed.

### Implementation Details

**Thread-Safe Spinlock**: Our `KSpinLock` implementation uses atomic compare-and-swap operations with proper memory ordering:
- `Acquire` ordering when acquiring the lock ensures no memory operations can be reordered before the lock acquisition
- `Release` ordering when releasing the lock ensures all previous memory operations complete before the lock is released
- The spinlock uses `core::hint::spin_loop()` for CPU-friendly busy waiting

**Screen Buffer Management**: The system maintains separate buffers for each virtual screen while only one physical buffer is displayed:
- Virtual screens store their content independently
- When switching screens, the new screen's buffer is copied to the physical VGA buffer
- This allows for instant screen switching with preserved content

**Hardware Cursor Control**: Direct VGA cursor control through port I/O:
- Uses VGA controller ports `0x3D4` and `0x3D5` to set cursor position
- Calculates linear cursor position from row/column coordinates
- Updates cursor position immediately when switching screens or moving within a screen

**Memory Safety**: Despite using unsafe code for hardware access and global state, the implementation maintains memory safety through:
- Proper initialization patterns with `MaybeUninit`
- Atomic operations for shared state
- Careful pointer management for VGA buffer access

### Key Features

**Multiple Virtual Screens**: Support for up to 3 concurrent virtual screens, each maintaining independent state and content.

**Instant Screen Switching**: Fast screen switching through keyboard shortcuts (Ctrl+Arrow keys) with immediate buffer swapping and cursor repositioning.

**Independent Screen State**: Each screen maintains its own cursor position, content, and can be written to independently of the currently active screen.

**Thread-Safe Operations**: All screen operations are protected by spinlocks, ensuring safe concurrent access in a potential multi-threaded environment.

**Hardware Integration**: Direct VGA hardware control for cursor positioning and buffer management, demonstrating low-level hardware interaction.

### Screen Switching Workflow

The screen switching process involves several coordinated steps:

1. **Input Detection**: Keyboard handler detects Ctrl+Arrow key combinations
2. **Screen Validation**: System checks if target screen exists and is available
3. **Buffer Swap**: Active screen's buffer is copied to physical VGA memory
4. **Cursor Update**: Hardware cursor is repositioned to match new screen's cursor position
5. **State Update**: Global active screen ID is updated to reflect the change