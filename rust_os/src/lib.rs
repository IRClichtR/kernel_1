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
                keyboard::KeyEvents ::Character(c) => {
                    printk!(LogLevel::Info, "Key pressed: '{}'\n", c);
                }
                // special keys
                keyboard::KeyEvents::Special(special_key) => {
                    match special_key {
                        keyboard::KeyEvents::ArrowUp => {
                            keyboard::move_cursor_up();
                            printk!(LogLevel::Info, "Arrow Up pressed\n");
                        }
                        keyboard::KeyEvents::ArrowDown => {
                            keyboard::move_cursor_down();
                            printk!(LogLevel::Info, "Arrow Down pressed\n");
                        }
                        keyboard::KeyEvents::ArrowLeft => {
                            keyboard::move_cursor_left();
                            printk!(LogLevel::Info, "Arrow Left pressed\n");
                        }
                        keyboard::KeyEvents::ArrowRight => {
                            keyboard::move_cursor_right();
                            printk!(LogLevel::Info, "Arrow Right pressed\n");
                        }
                        keyboard::KeyEvents::Home => {
                            keyboard::move_cursor_home();
                            printk!(LogLevel::Info, "Home pressed\n");
                        }
                        keyboard::KeyEvents::End => {
                            keyboard::move_cursor_end();
                            printk!(LogLevel::Info, "End pressed\n");
                        }
                        keyboard::KeyEvents::BackSpace => {
                            keyboard::handle_backspace();
                            printk!(LogLevel::Info, "Backspace pressed\n");
                        }
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}