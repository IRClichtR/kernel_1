use crate::printk;
use core::arch::asm;

#[repr(C, packed)]
pub struct GdtDescriptor {
    pub limit: u16,
    pub base: u32,
}

//===================================== PRINT GDT INFO =====================================

// Reads the Global Descriptor Table Register (GDTR) to get the base and limit of the GDT
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

//================================= PRINT GDT DESCRIPTORS ==================================

pub fn analyse_gdt_entry(gdt_base: u32, index: usize) {
    let descriptor_addr = gdt_base + (index * 8) as u32;

    unsafe {
        let ptr = descriptor_addr as *const u64;
        let descriptor = ptr.read();

        printk!(LogLevel::Info, "GDT Entry {}: {:#016x}\n", index, descriptor);

        let base_low = (descriptor & 0xFFFF) as u16;
        let base_mid = ((descriptor >> 16) & 0xFF) as u8;
        let base_high = ((descriptor >> 56) & 0xFF) as u8;
        let limit_low = ((descriptor >> 16) & 0xFFFF) as u16;
        let access = ((descriptor >> 40) & 0xFF) as u8;
        let flags = ((descriptor >> 52) & 0xF) as u8;

        let segment_base = (base_low as u32) | ((base_mid as u32) << 16) | ((base_high as u32) << 24);

        printk!(LogLevel::Info, "GDT Entry {}:\n", index);
        printk!(LogLevel::Info, "  Base: {:#010x}\n", segment_base);
        printk!(LogLevel::Info, "  Limit: {:#06x}\n", limit_low);
        printk!(LogLevel::Info, "  Access: {:#04x}\n", access);
        printk!(LogLevel::Info, "  Flags: {:#04x}\n", flags);
        printk!(LogLevel::Info, "  Segment Type: {}\n", if access & 0x10 != 0 { "Code" } else { "Data" });
        printk!(LogLevel::Info, "===========================================\n");
    }

}

pub fn dump_gdt_descriptors(gdt_desc: GdtDescriptor) {
    printk!(LogLevel::Info, "Dumping GDT Descriptors:\n");
    let gdt_base = gdt_desc.base;
    
    for i in 0..5 {
        analyse_gdt_entry(gdt_base, i);
    }
}