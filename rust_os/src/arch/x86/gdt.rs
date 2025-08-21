use crate::printk;
use core::arch::asm;

const KERNEL_DATA_SEGMENT_SELECTOR: u16 = 0x10;
#[allow(dead_code)]
const KERNEL_CODE_SEGMENT_SELECTOR: u16 = 0x08;
#[allow(dead_code)]
const USER_CODE_SEGMENT_SELECTOR: u16 = 0x08 | 0x03;

#[repr(C, packed)]
pub struct GdtDescriptor {
    pub limit: u16,
    pub base: u32,
}

#[repr(C, packed)]
struct SegmentDescriptor {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

const GDT_SIZE: usize = 7;
const GDT_ADDRESS: u32 = 0x0000_0800;

static mut GDT: *mut [SegmentDescriptor; GDT_SIZE] =
    GDT_ADDRESS as *mut [SegmentDescriptor; GDT_SIZE];

pub fn init() {
    unsafe {
        *GDT = [
            SegmentDescriptor::null(),                // Null segment
            SegmentDescriptor::new(0, 0xFFFFF, 0x9A), // Kernel code segment
            SegmentDescriptor::new(0, 0xFFFFF, 0x93), // Kernel data segment
            SegmentDescriptor::new(0, 0xFFFFF, 0x96), // Kernel stack segment
            SegmentDescriptor::new(0, 0xFFFFF, 0xFA), // User code segment
            SegmentDescriptor::new(0, 0xFFFFF, 0xF2), // User data segment
            SegmentDescriptor::new(0, 0xFFFFF, 0xF6), // User stack segment
        ];

        let gdtr = GdtDescriptor {
            #[allow(clippy::cast_possible_truncation)]
            limit: (size_of::<[SegmentDescriptor; GDT_SIZE]>() - 1) as u16,
            base: GDT_ADDRESS,
        };

        asm!(
            "lgdt [{}]",
            in(reg) &raw const gdtr,
            options(nostack, preserves_flags)
        );

        // Reload segment selectors
        asm!(
            "mov ax, {selector}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            options(nostack),
            selector = const KERNEL_DATA_SEGMENT_SELECTOR
        );

        asm!("push 0x08", "lea eax, [2f]", "push eax", "retf", "2:",);
    }
}

impl SegmentDescriptor {
    const fn new(base: u32, limit: u32, access: u8) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | 0xC0,
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }
}

// //===================================== PRINT GDT INFO =====================================

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

// //================================= PRINT GDT DESCRIPTORS ==================================

// pub fn analyse_gdt_entry(gdt_base: u32, index: usize) {
//     let descriptor_addr = gdt_base + (index * 8) as u32;

//     unsafe {
//         let ptr = descriptor_addr as *const u64;
//         let descriptor = ptr.read();

//         printk!(LogLevel::Info, "GDT Entry {}: {:#016x}\n", index, descriptor);

//         let base_low = (descriptor & 0xFFFF) as u16;
//         let base_mid = ((descriptor >> 16) & 0xFF) as u8;
//         let base_high = ((descriptor >> 56) & 0xFF) as u8;
//         let limit_low = ((descriptor >> 16) & 0xFFFF) as u16;
//         let access = ((descriptor >> 40) & 0xFF) as u8;
//         let flags = ((descriptor >> 52) & 0xF) as u8;

//         let segment_base = (base_low as u32) | ((base_mid as u32) << 16) | ((base_high as u32) << 24);

//         printk!(LogLevel::Info, "GDT Entry {}:\n", index);
//         printk!(LogLevel::Info, "  Base: {:#010x}\n", segment_base);
//         printk!(LogLevel::Info, "  Limit: {:#06x}\n", limit_low);
//         printk!(LogLevel::Info, "  Access: {:#04x}\n", access);
//         printk!(LogLevel::Info, "  Flags: {:#04x}\n", flags);
//         printk!(LogLevel::Info, "  Segment Type: {}\n", if access & 0x10 != 0 { "Code" } else { "Data" });
//         printk!(LogLevel::Info, "===========================================\n");
//     }

// }

// pub fn dump_gdt_descriptors(gdt_desc: GdtDescriptor) {
//     printk!(LogLevel::Info, "Dumping GDT Descriptors:\n");
//     let gdt_base = gdt_desc.base;
    
//     for i in 0..5 {
//         analyse_gdt_entry(gdt_base, i);
//     }
// }