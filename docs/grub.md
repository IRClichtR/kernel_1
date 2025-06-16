How GRUB Works with Assembly and a Custom Kernel

    GRUB is a pre-existing bootloader — you don't need to build it yourself. You just use it to load your kernel.

    When you write your own operating system or kernel, the first thing GRUB looks for is a multiboot header inside your kernel binary.

    That multiboot header is a small block of data (usually written in ASM) that tells GRUB how to load your kernel into memory and where to start executing.

    GRUB uses a configuration file (grub.cfg) where you define entries for different kernels. Example:

    menuentry "My Kernel" {
        multiboot /boot/my_kernel.bin
        boot
    }

    When the machine starts:

        GRUB launches and reads grub.cfg

        It finds your kernel binary, verifies the multiboot header

        GRUB loads your kernel and jumps to the address specified in that header

        Your kernel’s ASM code starts executing and can later call a main function (typically in C)

This allows you to use GRUB as a reliable bridge between your custom low-level code and the system’s hardware.
