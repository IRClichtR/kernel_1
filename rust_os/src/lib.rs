#![no_std]
#![no_main]
pub mod drivers;
pub mod printk;
pub mod arch;
pub mod screen;
pub mod kspin_lock;
pub mod command;

use core::panic::PanicInfo;
use crate::drivers::keyboard::{self, listen_to_keyboard_events};
use crate::screen::global::{init_screen_manager, screen_manager};
use crate::screen::screen::Writer;
use crate::command::{init_command_handler, command_handler};
// use crate::arch::x86::gdt::{read_gdtr, analyse_gdt_entry};
// use crate::arch::x86::gdt::read_gdtr;
use crate::arch::x86::gdt;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    gdt::init_gdt();
    init_screen_manager();
    init_command_handler(); 
    
    keyboard::init_keyboard();

    loop {
        listen_to_keyboard_events();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}