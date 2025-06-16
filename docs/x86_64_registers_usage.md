
# x86_64 Registers: Usage and Size Breakdown

In x86_64 assembly, understanding the relationship between 64-bit, 32-bit, 16-bit, and 8-bit registers is crucial. This document outlines how these registers work and their usage in different contexts.

## Table of Registers

| Name   | Size   | Description / Part of the Register |
|--------|--------|-----------------------------------|
| `rax`  | 64 bits| Full register (e.g., result of `syscall`) |
| `eax`  | 32 bits| Lower 32 bits of `rax` |
| `ax`   | 16 bits| Lower 16 bits of `eax` |
| `al`   | 8 bits | Lower 8 bits of `ax` (bits 0-7) |
| `ah`   | 8 bits | Higher 8 bits of `ax` (bits 8-15) |

This pattern repeats for other registers:

| 64-bit  | 32-bit  | 16-bit  | 8-bit low | 8-bit high |
|---------|---------|---------|-----------|------------|
| `rax`   | `eax`   | `ax`    | `al`      | `ah`       |
| `rbx`   | `ebx`   | `bx`    | `bl`      | `bh`       |
| `rcx`   | `ecx`   | `cx`    | `cl`      | `ch`       |
| `rdx`   | `edx`   | `dx`    | `dl`      | `dh`       |
| `rsi`   | `esi`   | `si`    | `sil`     | –          |
| `rdi`   | `edi`   | `di`    | `dil`     | –          |
| `rsp`   | `esp`   | `sp`    | `spl`     | –          |
| `rbp`   | `ebp`   | `bp`    | `bpl`     | –          |
| `r8`–`r15` | `r8d`–`r15d` | `r8w`–`r15w` | `r8b`–`r15b` | – |

## Key Concepts

- Assigning to `eax` **automatically clears** the upper 32 bits of `rax`.
- You cannot directly move 64-bit values into 32-bit registers (e.g., `mov ecx, rsi` is invalid), but you can move the lower 32 bits using `mov ecx, esi`.
- In the `int 0x80` system call (32-bit system call interface), only the 32-bit registers (`eax`, `ebx`, etc.) are used, even on 64-bit systems.
- In the `syscall` instruction (for 64-bit systems), use 64-bit registers (`rax`, `rdi`, `rsi`, `rdx`, etc.).

## When to Use Which Registers?

| Context        | Use These Registers              |
|----------------|----------------------------------|
| `int 0x80`     | `eax`, `ebx`, `ecx`, `edx`, etc. |
| `syscall` (x86_64) | `rax`, `rdi`, `rsi`, `rdx`, etc. |

---

This document summarizes the basic concepts and practical usage of registers in x86_64 assembly. It is helpful to keep these distinctions in mind when working with system calls, especially when dealing with transitions between 32-bit and 64-bit modes in the Linux environment.
