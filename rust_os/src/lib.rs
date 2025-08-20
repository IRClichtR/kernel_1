#![no_std]
#![no_main]
pub mod drivers;
pub mod printk;
pub mod arch;
pub mod screen;
pub mod kspin_lock;
pub mod command;

use core::panic::PanicInfo;
use crate::drivers::keyboard;
use crate::screen::global::{init_screen_manager, screen_manager};
use crate::screen::screen::Writer;
use crate::command::{init_command_handler, command_handler};

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init_screen_manager();
    init_command_handler(); 
    
    keyboard::init_keyboard();

    loop {
        // Poll keyboard for input
        if let Some(key_event) = keyboard::poll_keyboard() {
            match key_event {
                // Character input
                keyboard::KeyEvents::Character(c) => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.add_char(c as u8, &mut manager);
                }
                
                // Arrow key navigation
                keyboard::KeyEvents::ArrowUp => {
                    // Could be used for command history in the future
                    keyboard::move_cursor_up();
                }
                keyboard::KeyEvents::ArrowDown => {
                    // Could be used for command history in the future
                    keyboard::move_cursor_down();
                }
                keyboard::KeyEvents::ArrowLeft => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.move_cursor_left(&mut manager);
                }
                keyboard::KeyEvents::ArrowRight => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.move_cursor_right(&mut manager);
                }
                keyboard::KeyEvents::Home => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.move_cursor_home(&mut manager);
                }
                keyboard::KeyEvents::End => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.move_cursor_end(&mut manager);
                }
                keyboard::KeyEvents::BackSpace => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.backspace(&mut manager);
                }
                keyboard::KeyEvents::Delete => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.delete_char(&mut manager);
                }
                
                keyboard::KeyEvents::Enter => {
                    let mut manager = screen_manager().lock();
                    
                    // Add newline to move to next line
                    let mut writer = Writer::new(&mut manager.screen);
                    writer.write_byte(b'\n');
                    manager.flush_to_physical();
                    manager.update_cursor();
                    
                    // Release manager lock before executing command
                    drop(manager);
                    
                    // Execute the command
                    {
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.execute_command();
                    }
                    
                    // Show prompt again and set prompt position
                    {
                        let mut manager = screen_manager().lock();
                        // Write the prompt
                        let mut writer = Writer::new(&mut manager.screen);
                        writer.write_byte(b'>');
                        writer.write_byte(b' ');
                        
                        // Get current cursor position for prompt (after writing "> ")
                        let prompt_row = manager.screen.row_position;
                        let prompt_col = manager.screen.column_position;
                        
                        manager.flush_to_physical();
                        manager.update_cursor();
                        
                        // Update prompt position in command handler
                        drop(manager);
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.set_prompt_position(prompt_row, prompt_col);
                    }
                }
                
                // Remove screen switching events - they are no longer supported
                keyboard::KeyEvents::SwitchScreenLeft | keyboard::KeyEvents::SwitchScreenRight => {
                    // Ignore screen switching events
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}