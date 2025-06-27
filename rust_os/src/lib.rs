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

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init_screen_manager();
    
    // Test basic screen functionality
    {
        let mut manager = screen_manager().lock();
        
        // Create a second screen
        if let Some(_screen_id) = manager.create_screen() {
            
            // Write directly to screen 0
            if let Some(screen) = &mut manager.screens[0] {
                let mut writer = Writer::new(screen);
                write!(writer, "Screen 0\n").unwrap();
            }
            
            // Switch to screen 1 and write
            if manager.switch_screen(1) {
                if let Some(screen) = &mut manager.screens[1] {
                    let mut writer = Writer::new(screen);
                    write!(writer, "Screen 1\n").unwrap();
                }
            }
            
            // Switch back to screen 0
            manager.switch_screen(0);
        }
    }
    
    // === PHASE 3: TESTING SCREEN-AWARE PRINTK ===
    
    // Test 1: printk on screen 0 (default)
    printk!(LogLevel::Info, "=== Screen-Aware printk Test ===\n");
    printk!(LogLevel::Info, "Test 1: Writing to screen 0 (default)\n");
    printk!(LogLevel::Warn, "This warning should appear on screen 0\n");
    printk!(LogLevel::Error, "This error should appear on screen 0\n");
    printk!(LogLevel::Debug, "Debug message on screen 0\n");
    printk!(LogLevel::Notice, "Notice message on screen 0\n");
    printk!("Default level message on screen 0\n");
    
    printk!(LogLevel::Info, "=== Screen-Aware printk Test Complete ===\n");
    printk!(LogLevel::Info, "Use Ctrl+Alt+Left/Right to switch screens\n");
    printk!(LogLevel::Info, "All printk output should appear on the active screen\n");
    
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
                // screen switching
                keyboard::KeyEvents::SwitchScreenLeft => {
                    let mut manager = screen_manager().lock();
                    let current_screen = manager.active_screen_id;
                    let new_screen = if current_screen == 0 { 1 } else { 0 };
                    if !manager.switch_screen(new_screen) {
                        printk!(LogLevel::Critical, "Error switching to screen {}\n", new_screen);
                    } else {
                        printk!(LogLevel::Info, "Switched to screen {}\n", new_screen);
                    }
                }
                keyboard::KeyEvents::SwitchScreenRight => {
                    let mut manager = screen_manager().lock();
                    let current_screen = manager.active_screen_id;
                    let new_screen = if current_screen == 0 { 1 } else { 0 };
                    if !manager.switch_screen(new_screen) {
                        printk!(LogLevel::Critical, "Error switching to screen {}\n", new_screen);
                    } else {
                        printk!(LogLevel::Info, "Switched to screen {}\n", new_screen);
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