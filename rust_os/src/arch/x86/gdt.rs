use core::arch::asm;

static GDT_SIZE: usize = 7;

#[allow(dead_code)]
struct GdtRegister {
    limit: u16,
    base: u32,
}

#[allow(dead_code)]
struct SegmentDescriptor {
    limit_low: u16, //bit 0-15
    base_low: u16,  //bit 0-15
    base_middle: u8, //bit 16-23
    access: u8,     //bit 24-31
    granularity: u8, //bit 32-39
    base_high: u8,  //bit 40-47
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

static mut GDT: [SegmentDescriptor; GDT_SIZE] = [
    SegmentDescriptor::null(),                // Null segment
    SegmentDescriptor::new(0, 0xFFFFF, 0x9A), // Kernel code segment
    SegmentDescriptor::new(0, 0xFFFFF, 0x93), // Kernel data segment
    SegmentDescriptor::new(0, 0xFFFFF, 0x96), // Kernel stack segment
    SegmentDescriptor::new(0, 0xFFFFF, 0xFA), // User code segment
    SegmentDescriptor::new(0, 0xFFFFF, 0xF2), // User data segment
    SegmentDescriptor::new(0, 0xFFFFF, 0xF6), // User stack segment
];

pub fn init_gdt() {
    let gdt_register = GdtRegister {
        limit: (core::mem::size_of::<[SegmentDescriptor; GDT_SIZE]>() - 1) as u16,
        base: core::ptr::addr_of!(GDT).cast::<u32>() as u32,
    };

    unsafe {
        asm!("lgdt [{}]", in(reg) &gdt_register, options(nostack));

        // Reload segment selectors
        asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            options(nostack)
        );

        // Reload code segment
        asm!("push 0x08", "lea eax, [2f]", "push eax", "retf", "2:",); 
    }
}