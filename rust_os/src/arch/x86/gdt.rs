use crate::arch::x86::port::{inb, outb};

#[repr(C, packed)]
pub struct GdtDescriptor {
    pub limit: u16,
    pub base: u32,
}

#[inline(always)]
pub fn read_gdtr() -> GdtDescriptor {
    let mut gdtr = GdtDescriptor {
        limit: 0,
        base: 0,
    };

    unsafe {
        asm!(
            "sgdt [{}]",
            in(reg) &mut gdtr,
            options(nostack, preserves_flags)
        )
    }
    
    gdtr
}