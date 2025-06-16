#![no_std]
#![no_main]

pub mod drivers;
pub mod printk;

use core::panic::PanicInfo;
// use crate::drivers::keyboard::process_keyboard;
use crate::printk::printk::LogLevel;
// use crate::arch::x86::pic::{init_pics, enable_keyboard_interrupt};

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    printk!(LogLevel::Info,"Strarting Kernel...\n");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}