; Constants for the multiboot header
%define ALIGN 1<<0                  ; align the header to 1
%define MEMINFO 1<<1                ; provide memory map. This part is important to know witch part of the memory is free and witch part is used by the kernel
%define FLAGS (ALIGN | MEMINFO)     ; flags for the multiboot header
%define MAGIC 0x1BADB002            ; magic number for multiboot1 multiboot2 is 0x36D76289. Let's the bootloaeder find the header
%define CHECKSUM (-(MAGIC + FLAGS)) ; checksum for the multiboot header to prove that the header is valid

; External symbol for the main function
extern kernel_main                  ; this is the main function of the kernel. It is defined in another file

; The multiboot header is a special structure that the bootloader uses to find the kernel and load it into memory
; The header is 32 bytes long and contains the following fields:
section .multiboot                   ; new seciton in the binary contains the multiboot header information
align 4                              ; ensure the header is aligned to 4 bytes. Required by the multiboot specification
dd MAGIC                             ; places the magic number value into the header
dd FLAGS                             ; places the flags value into the header
dd CHECKSUM                          ; Adds a checksum value that must make the first three 32-bit values sum to zero

; Stack definition: the multiboot standard does not define a stack, so we need to define it ourselves
; The stack is defined in the .bss section, which is uninitialized data. The stack grows downwards, so we need to define the bottom of the stack first
; The stack is 16 KiB in size, which is enough for most applications. The stack pointer will be set to the top of the stack

section .bss                ; new section in the binary contains uninitialized data
align 16                    ; ensure the stack is aligned to 16 bytes. This is required by the x86_64 ABI
stack_bottom:               ; this is the bottom of the stack
    resb 16384              ; reserve 16 KiB of space for the stack. This is the size of the stack
stack_top:                  ; this is the top of the stack
; The stack pointer will be set to the top of the stack

section .text               ; new section in the binary contains code
global _start               ; make the _start label global so the linker can find it
_start:                     ; this is the entry point of the kernel | tell the linker where the kernel starts
    ; Set up the stack pointer
    mov esp, $stack_top     ; set the stack pointer to the top of the stack

    ; Call the main function
    call kernel_main        ; call the main function don't name it _clestart to avoid confusion with the _start label

    ; Infinite loop to prevent the kernel from exiting
    cli                     ; disable interrupts
    hlt                     ; halt the CPU until the next interrupt
    jmp $                   ; jump to the beginning of the loop
    ; This is a simple infinite loop that will keep the kernel running
