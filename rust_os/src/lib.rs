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
use crate::command::CommandHandler;
use crate::kspin_lock::kspin_lock::KSpinLock;

// Global command handler for screen 1
static COMMAND_HANDLER: KSpinLock<CommandHandler> = KSpinLock::new(CommandHandler::new());

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init_screen_manager();
    
    {
        let mut manager = screen_manager().lock();
    
        if let Some(_screen_id) = manager.create_screen() {            
            if manager.switch_screen(1) {
                if let Some(screen) = &mut manager.screens[1] {
                    let mut writer = Writer::new(screen);
                    write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
                    write!(writer, "\n").unwrap();
                    write!(writer, "Type 'help' for available commands.\n").unwrap();
                    write!(writer, "> ").unwrap();
                }
            }

            manager.switch_screen(0);
        }
    }
    
    keyboard::init_keyboard();

    loop {
        // Poll keyboard for input
        if let Some(key_event) = keyboard::poll_keyboard() {
            match key_event {
                // chars - now uses screen manager
                keyboard::KeyEvents::Character(c) => {
                    let mut manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        // Screen 1: Add character to command buffer
                        let mut cmd_handler = COMMAND_HANDLER.lock();
                        cmd_handler.add_char(c as u8);
                    }
                    
                    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                        let mut writer = Writer::new(active_screen);
                        writer.write_byte(c as u8);
                    }
                    manager.flush_to_physical();
                    manager.update_cursor();
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
                    let mut manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        // Screen 1: Remove character from command buffer
                        let mut cmd_handler = COMMAND_HANDLER.lock();
                        cmd_handler.backspace();
                    }
                    
                    keyboard::handle_backspace();
                }
                // keyboard::KeyEvents::Delete => {
                //     let mut manager = screen_manager().lock();
                //     let active_screen_id = manager.active_screen_id;
                    
                //     // Note: Delete key deletes character at cursor position, not behind it
                //     // For now, we'll treat it similar to backspace for the command buffer
                //     if active_screen_id == 1 {
                //         // Screen 1: Remove character from command buffer (similar to backspace)
                //         let mut cmd_handler = COMMAND_HANDLER.lock();
                //         cmd_handler.backspace();
                //     }
                    
                //     // Release manager lock before calling handle_delete
                //     drop(manager);
                    
                //     keyboard::handle_delete();
                // }


                keyboard::KeyEvents::Delete => {
                    // Handle command buffer first (if needed)
                    {
                        let manager = screen_manager().lock();
                        let active_screen_id = manager.active_screen_id;
                        
                        if active_screen_id == 1 {
                            drop(manager);  // Release before acquiring command handler lock
                            let mut cmd_handler = COMMAND_HANDLER.lock();
                            cmd_handler.backspace();
                            drop(cmd_handler);
                        }
                    }
                    
                    // Now handle the delete operation
                    keyboard::handle_delete();
                }


                // pub fn handle_delete() {
                //     let mut manager = screen_manager().lock();
                //     let active_screen_id = manager.active_screen_id;
                //     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                //         active_screen.write_byte_at(
                //             active_screen.row_position,
                //             active_screen.column_position,
                //             b' '
                //         );
                //     }
                //     manager.flush_to_physical();
                //     manager.update_cursor();
                // }
                keyboard::KeyEvents::Enter => {
                    let mut manager = screen_manager().lock();
                    let active_screen_id = manager.active_screen_id;
                    
                    if active_screen_id == 1 {
                        // Screen 1: Execute command
                        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                            let mut writer = Writer::new(active_screen);
                            writer.write_byte(b'\n');
                        }
                        manager.flush_to_physical();
                        manager.update_cursor();
                        
                        // Release manager lock before executing command
                        drop(manager);
                        
                        // Execute the command
                        let mut cmd_handler = COMMAND_HANDLER.lock();
                        cmd_handler.execute_command();
                        drop(cmd_handler);
                        
                        // Show prompt again
                        let mut manager = screen_manager().lock();
                        if let Some(screen) = &mut manager.screens[1] {
                            let mut writer = Writer::new(screen);
                            write!(writer, "> ").unwrap();
                        }
                        manager.flush_to_physical();
                        manager.update_cursor();
                    } else {
                        // Other screens: Just add newline
                        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                            let mut writer = Writer::new(active_screen);
                            writer.write_byte(b'\n');
                        }
                        manager.flush_to_physical();
                        manager.update_cursor();
                    }
                }
                keyboard::KeyEvents::SwitchScreenLeft => {
                    let switch_successful = {
                        let mut manager = screen_manager().lock();
                        let current_screen = manager.active_screen_id;
                        let new_screen = if current_screen == 0 { 1 } else { 0 };
                        manager.switch_screen(new_screen)
                    };
                    
                    if !switch_successful {
                        printk!(LogLevel::Critical, "Fatal error switching the screen\n");
                    }
                }
                keyboard::KeyEvents::SwitchScreenRight => {
                    let switch_successful = {
                        let mut manager = screen_manager().lock();
                        let current_screen = manager.active_screen_id;
                        let new_screen = if current_screen == 0 { 1 } else { 0 };
                        manager.switch_screen(new_screen)
                    };
                    
                    if !switch_successful {
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