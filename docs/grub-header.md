# Project Documentation: GRUB and Bootable Kernel

## 1. Project Overview

This project demonstrates the creation of a minimal operating system kernel that can be booted via GRUB. The deliverables include:

- A **bootable kernel using GRUB**
- A **boot sector written in Assembly**
- A **basic kernel library** with essential functions and types
- A mechanism to **display text to the screen**
- A working "**Hello, world**" printed at boot

---

## 2. GRUB: The Bootloader

**GRUB (GRand Unified Bootloader)** is a flexible bootloader that facilitates the loading of different operating systems. It is used here to load our custom kernel.

---

## 3. Multiboot2 Specification

To ensure compatibility with GRUB, our kernel must conform to the **Multiboot2 specification**.

### Key Objectives:

- **Boot from various sources**: GRUB can load the OS image from hard drives, floppy disks, or over a network.
- **No special file format**: OS images are compiled as standard 32-bit executables.
- **Mode switching**: GRUB handles the switch to 32-bit protected mode, easing kernel design.
- **Pass configuration info**: GRUB allows users to pass dynamic boot-time information to the OS.

---

## 4. Multiboot2 Header Structure

The Multiboot2 header must begin with the following fields, defined in native endianness:

| Offset | Type | Field Name       | Description                                           |
|--------|------|------------------|-------------------------------------------------------|
| 0      | u32  | `magic`          | Must be `0xE85250D6`                                  |
| 4      | u32  | `architecture`   | `0` for x86 protected mode                            |
| 8      | u32  | `header_length`  | Total size of the header, including all tags          |
| 12     | u32  | `checksum`       | Must satisfy: `magic + architecture + header_length + checksum == 0` |

Additional fields (`tags`) define memory requirements, framebuffer settings, and other boot-time parameters.

---

## 5. Why Use GRUB?

- **Simplifies development**: No need for custom bootloaders.
- **Standardized format**: Allows use of common development tools (`nm`, disassemblers).
- **Reusability**: GRUB reclaims memory after booting, unlike permanent OS code.