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
// use crate::arch::x86::gdt::{read_gdtr, analyse_gdt_entry};
use crate::arch::x86::gdt::read_gdtr;
use crate::arch::x86::gdt;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init_screen_manager();
    init_command_handler(); 
    gdt::init();
    
    keyboard::init_keyboard();
    let gdt_desc = read_gdtr();
    let limit = gdt_desc.limit as usize;
    let gdt_base = gdt_desc.base as usize;
    // printk!(LogLevel::Info, "GDT Base: {:#010x}, Limit: {:#06x}\n", gdt_base, limit);

    loop {
        if let Some(key_event) = keyboard::poll_keyboard() {
            match key_event {
                keyboard::KeyEvents::Character(c) => {
                    let mut manager = screen_manager().lock();
                    let mut cmd_handler = command_handler().lock();
                    cmd_handler.add_char(c as u8, &mut manager);
                }
                
                keyboard::KeyEvents::ArrowUp => {
                    keyboard::move_cursor_up();
                }
                keyboard::KeyEvents::ArrowDown => {
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
                    
                    if let Some(screen) = manager.get_screen_mut(2) {
                        let mut writer = Writer::new(screen);
                        writer.write_byte(b'\n');
                    }
                    
                    if manager.get_active_screen_id() == 2 {
                        manager.flush_to_physical();
                        manager.update_cursor();
                    }
                    
                    drop(manager);
                    
                    {
                        let mut cmd_handler = command_handler().lock();
                        cmd_handler.execute_command();
                    }
                    
                    {
                        let mut manager = screen_manager().lock();
                        if let Some(screen) = manager.get_screen_mut(2) {
                            let mut writer = Writer::new(screen);
                            writer.write_byte(b'>');
                            writer.write_byte(b' ');
                            
                            let prompt_row = screen.row_position;
                            let prompt_col = screen.column_position;
                            
                            if manager.get_active_screen_id() == 2 {
                                manager.flush_to_physical();
                                manager.update_cursor();
                            }
                            
                            drop(manager);
                            let mut cmd_handler = command_handler().lock();
                            cmd_handler.set_prompt_position(prompt_row, prompt_col);
                        }
                    }
                }
                
                keyboard::KeyEvents::SwitchScreenLeft => {
                    let mut manager = screen_manager().lock();
                    let current_screen = manager.get_active_screen_id();
                    let new_screen = if current_screen == 1 { 2 } else { 1 };
                    let switch_successful = manager.switch_screen(new_screen);
                    
                    if switch_successful {
                        drop(manager);
                    } else {
                        drop(manager);
                        printk!(LogLevel::Critical, "Fatal error switching the screen\n");
                    }
                }
                keyboard::KeyEvents::SwitchScreenRight => {
                    let mut manager = screen_manager().lock();
                    let current_screen = manager.get_active_screen_id();
                    let new_screen = if current_screen == 1 { 2 } else { 1 };
                    let switch_successful = manager.switch_screen(new_screen);
                    
                    if switch_successful {
                        drop(manager);
                    } else {
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