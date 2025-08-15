#![no_std]
#![no_main]
pub mod drivers;
pub mod printk;
pub mod arch;
pub mod screen;
pub mod kspin_lock;
pub mod command;

use core::panic::PanicInfo;
use core::fmt::Write;
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
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        drop(manager);
                        
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.add_char(c as u8);
                    } else {
                        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                            let mut writer = Writer::new(active_screen);
                            writer.write_byte(c as u8);
                        }
                        manager.flush_to_physical();
                        manager.update_cursor();
                    }
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
                    let manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        // Screen 1: Move cursor within command buffer
                        drop(manager);
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.move_cursor_left();
                    } else {
                        // Other screens: Regular cursor movement
                        drop(manager);
                        keyboard::move_cursor_left();
                    }
                }
                keyboard::KeyEvents::ArrowRight => {
                    let manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        drop(manager);
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.move_cursor_right();
                    } else {
                        drop(manager);
                        keyboard::move_cursor_right();
                    }
                }
                keyboard::KeyEvents::Home => {
                    let manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        drop(manager);
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.move_cursor_home();
                    } else {
                        drop(manager);
                        keyboard::move_cursor_home();
                    }
                }
                keyboard::KeyEvents::End => {
                    let manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        drop(manager);
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.move_cursor_end();
                    } else {
                        drop(manager);
                        keyboard::move_cursor_end();
                    }
                }
                keyboard::KeyEvents::BackSpace => {
                    let manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        drop(manager);
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.backspace();
                    } else {
                        drop(manager);
                        keyboard::handle_backspace();
                    }
                }
                keyboard::KeyEvents::Delete => {
                    let manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        drop(manager);
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.delete_char();
                    }
                }
                
                keyboard::KeyEvents::Enter => {
                    let mut manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        // Add newline to move to next line
                        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                            let mut writer = Writer::new(active_screen);
                            writer.write_byte(b'\n');
                        }
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
                            if let Some(screen) = &mut manager.screens[1] {
                                // Write the prompt
                                let mut writer = Writer::new(screen);
                                writer.write_byte(b'>');
                                writer.write_byte(b' ');
                                
                                // Get current cursor position for prompt (after writing "> ")
                                let prompt_row = screen.row_position;
                                let prompt_col = screen.column_position;
                                
                                manager.flush_to_physical();
                                manager.update_cursor();
                                
                                // Update prompt position in command handler
                                drop(manager);
                                let mut cmd_handler = command_handler().lock();
                                cmd_handler.set_prompt_position(prompt_row, prompt_col);
                            }
                            else {
                                manager.flush_to_physical();
                                manager.update_cursor();
                            }
                        }
                    } else {
                        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                            let mut writer = Writer::new(active_screen);
                            writer.write_byte(b'\n');
                        }
                        manager.flush_to_physical();
                        manager.update_cursor();
                    }
                }
                
                // Screen switching
                keyboard::KeyEvents::SwitchScreenLeft => {
                    let mut manager = screen_manager().lock();
                    let current_screen = manager.active_screen_id;
                    let new_screen = if current_screen == 0 { 1 } else { 0 };
                    let switch_successful = manager.switch_screen(new_screen);
                    
                    if !switch_successful {
                        drop(manager);
                        printk!(LogLevel::Critical, "Fatal error switching the screen\n");
                    }
                }
                keyboard::KeyEvents::SwitchScreenRight => {
                    let mut manager = screen_manager().lock();
                    let current_screen = manager.active_screen_id;
                    let new_screen = if current_screen == 0 { 1 } else { 0 };
                    let switch_successful = manager.switch_screen(new_screen);
                    
                    if !switch_successful {
                        drop(manager);
                        printk!(LogLevel::Critical, "Fatal error switching the screen\n");
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