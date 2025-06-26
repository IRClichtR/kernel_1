#![no_std]
#![no_main]
pub mod drivers;
pub mod printk;
pub mod arch;
pub mod vga_buffer;
pub mod screen;
pub mod kspin_lock;
use core::panic::PanicInfo;
use core::fmt::Write;
use crate::printk::printk::LogLevel;
use crate::drivers::keyboard;
use crate::screen::global::{init_screen_manager, screen_manager};
use crate::screen::screen::Writer;
use crate::printk::printk::{set_printk_screen, get_printk_screen};

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init_screen_manager();
    
    // Test screen management
    {
        let mut manager = screen_manager().lock();
        
        // Create a second screen
        if let Some(screen_id) = manager.create_screen() {
            printk!(LogLevel::Info, "Created screen {}\n", screen_id);
            
            // Set printk to write to screen 0
            set_printk_screen(0);
            printk!(LogLevel::Info, "This message goes to screen 0\n");
            
            // Switch to screen 1 and set printk target
            if manager.switch_screen(1) {
                set_printk_screen(1);
                printk!(LogLevel::Info, "This message goes to screen 1\n");
                
                // Write directly to screen 1
                if let Some(screen) = &mut manager.screens[1] {
                    let mut writer = Writer::new(screen);
                    write!(writer, "Direct write to screen 1\n").unwrap();
                }
            }
            
            // Switch back to screen 0
            manager.switch_screen(0);
            set_printk_screen(0);
            printk!(LogLevel::Info, "Back to screen 0\n");
        }
    }
    
    // // Init keyboard
    // keyboard::init_keyboard();
    
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
                // editing keys
                keyboard::KeyEvents::BackSpace => {
                    keyboard::handle_backspace();
                }
                keyboard::KeyEvents::Delete => {
                    keyboard::handle_delete();
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