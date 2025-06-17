#![no_std]
#![no_main]

pub mod drivers;
pub mod printk;
pub mod arch;
pub mod vga_buffer;

use core::panic::PanicInfo;
use crate::printk::printk::LogLevel;
use crate::drivers::keyboard;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    printk!(LogLevel::Info,"Starting Kernel...\n");
    // Init keyboard
    keyboard::init_keyboard();
    loop {
        // Poll keyboard for input
        if let Some(key) = keyboard::poll_keyboard() {
            printk!(LogLevel::Info, "{}", key);
        } 
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}