#![no_std]
#![no_main]

pub mod drivers;
pub mod printk;
pub mod arch;
pub mod vga_buffer;

use core::panic::PanicInfo;
use crate::printk::printk::LogLevel;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    printk!(LogLevel::Info,"Starting Kernel...\n");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}