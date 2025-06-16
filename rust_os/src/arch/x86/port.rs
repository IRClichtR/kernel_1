// Read byte from I/O port
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let mut data: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") data,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    data
}

// Write a byte to an I/O port
#[inline]
pub unsafe fn outb(port: u16, data: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") data,
        options(nomem, nostack, preserves_flags)
    );
}

// Small delay by writing to an unused I/O port
#[inline]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}