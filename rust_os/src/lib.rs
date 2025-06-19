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
        if let Some(key_event) = keyboard::poll_keyboard() {
            match key_event {
                // chars
                keyboard::KeyEvents::Character(c) => {
                    vga_buffer::WRITER.lock().write_byte(c as u8);
                }
                // special keys
                keyboard::KeyEvents::ArrowUp => {
                    keyboard::move_cursor_up();
                }
                keyboard::KeyEvents::ArrowDown => {
                    keyboard::move_cursor_down();
                }
                keyboard::KeyEvents::ArrowLeft => {
                    keyboard::move_cursor_left();
                }
                keyboard::KeyEvents::ArrowRight => {
                    keyboard::move_cursor_right();
                }
                keyboard::KeyEvents::Home => {
                    keyboard::move_cursor_home();
                }
                keyboard::KeyEvents::End => {
                    keyboard::move_cursor_end();
                }
                // other keys
                keyboard::KeyEvents::BackSpace => {
                    keyboard::handle_backspace();
                }
                keyboard::KeyEvents::Enter => {
                    vga_buffer::WRITER.lock().new_line();
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}