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
    
    // Test 2: Switch to screen 1 and test printk
    {
        let mut manager = screen_manager().lock();
        if manager.switch_screen(1) {
            printk!(LogLevel::Info, "Test 2: Switched to screen 1\n");
            printk!(LogLevel::Warn, "This warning should appear on screen 1\n");
            printk!(LogLevel::Error, "This error should appear on screen 1\n");
            printk!(LogLevel::Debug, "Debug message on screen 1\n");
        }
    }
    
    // Test 3: Switch back to screen 0 and test printk
    {
        let mut manager = screen_manager().lock();
        if manager.switch_screen(0) {
            printk!(LogLevel::Info, "Test 3: Back to screen 0\n");
            printk!(LogLevel::Notice, "Notice: printk should work on screen 0 again\n");
            printk!(LogLevel::Debug, "Debug message on screen 0\n");
        }
    }
    
    // Test 4: Multiple rapid screen switches with printk
    for i in 0..3 {
        {
            let mut manager = screen_manager().lock();
            if manager.switch_screen(1) {
                printk!(LogLevel::Info, "Rapid test {}: Screen 1\n", i);
            }
        }
        
        {
            let mut manager = screen_manager().lock();
            if manager.switch_screen(0) {
                printk!(LogLevel::Info, "Rapid test {}: Screen 0\n", i);
            }
        }
    }
    
    // Test 5: Test all log levels on current screen
    printk!(LogLevel::Emergency, "Emergency message\n");
    printk!(LogLevel::Alert, "Alert message\n");
    printk!(LogLevel::Critical, "Critical message\n");
    printk!(LogLevel::Error, "Error message\n");
    printk!(LogLevel::Warn, "Warning message\n");
    printk!(LogLevel::Notice, "Notice message\n");
    printk!(LogLevel::Info, "Info message\n");
    printk!(LogLevel::Debug, "Debug message\n");
    printk!("Default level message\n");
    
    printk!(LogLevel::Info, "=== Screen-Aware printk Tests Complete ===\n");
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