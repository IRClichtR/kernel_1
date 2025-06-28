// #![no_std]
// #![no_main]
// pub mod drivers;
// pub mod printk;
// pub mod arch;
// pub mod screen;
// pub mod kspin_lock;
// use core::panic::PanicInfo;
// use core::fmt::Write;
// use crate::drivers::keyboard;
// use crate::screen::global::{init_screen_manager, screen_manager};
// use crate::screen::screen::Writer;

// #[no_mangle]
// pub extern "C" fn kernel_main() -> ! {
//     init_screen_manager();
    
//     {
//         let mut manager = screen_manager().lock();
    
//         if let Some(_screen_id) = manager.create_screen() {            
//             if manager.switch_screen(1) {
//                 if let Some(screen) = &mut manager.screens[1] {
//                     let mut writer = Writer::new(screen);
//                     write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
//                     write!(writer, "\n").unwrap();
//                 }
//             }

//             manager.switch_screen(0);
//         }
//     }
    
//     keyboard::init_keyboard();

//     loop {
//         // Poll keyboard for input
//         if let Some(key_event) = keyboard::poll_keyboard() {
//             match key_event {
//                 // chars - now uses screen manager
//                 keyboard::KeyEvents::Character(c) => {
//                     let mut manager = screen_manager().lock();
//                     let active_screen_id = manager.active_screen_id; // Store the ID first
//                     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//                         use crate::screen::screen::Writer;
//                         let mut writer = Writer::new(active_screen);
//                         writer.write_byte(c as u8);
//                     }
//                     manager.flush_to_physical();
//                     manager.update_cursor();
//                 }
                
//                 // special keys
//                 keyboard::KeyEvents::ArrowUp => {
//                     keyboard::move_cursor_up();
//                 }
//                 keyboard::KeyEvents::ArrowDown => {
//                     keyboard::move_cursor_down();
//                 }
//                 keyboard::KeyEvents::ArrowLeft => {
//                     keyboard::move_cursor_left();
//                 }
//                 keyboard::KeyEvents::ArrowRight => {
//                     keyboard::move_cursor_right();
//                 }
//                 keyboard::KeyEvents::Home => {
//                     keyboard::move_cursor_home();
//                 }
//                 keyboard::KeyEvents::End => {
//                     keyboard::move_cursor_end();
//                 }
                
//                 // editing keys
//                 keyboard::KeyEvents::BackSpace => {
//                     keyboard::handle_backspace();
//                 }
//                 keyboard::KeyEvents::Delete => {
//                     keyboard::handle_delete();
//                 }
                
//                 // enter key - now uses screen manager
//                 keyboard::KeyEvents::Enter => {
//                     let mut manager = screen_manager().lock();
//                     let active_screen_id = manager.active_screen_id; // Store the ID first
//                     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//                         use crate::screen::screen::Writer;
//                         let mut writer = Writer::new(active_screen);
//                         writer.write_byte(b'\n');  // This will trigger new_line logic in Writer
//                     }
//                     manager.flush_to_physical();
//                     manager.update_cursor();
//                 }
//                 keyboard::KeyEvents::SwitchScreenLeft => {
//                     let switch_successful = {
//                         let mut manager = screen_manager().lock();
//                         let current_screen = manager.active_screen_id;
//                         let new_screen = if current_screen == 0 { 1 } else { 0 };
//                         manager.switch_screen(new_screen)
//                     };
                    
//                     if !switch_successful {
//                         printk!(LogLevel::Critical, "Fatal error switching the screen\n");
//                     }
//                 }
//                 keyboard::KeyEvents::SwitchScreenRight => {
//                     let switch_successful = {
//                         let mut manager = screen_manager().lock();
//                         let current_screen = manager.active_screen_id;
//                         let new_screen = if current_screen == 0 { 1 } else { 0 };
//                         manager.switch_screen(new_screen)
//                     };
                    
//                     if !switch_successful {
//                         printk!(LogLevel::Critical, "Fatal error switching the screen\n");
//                     }
//                 }
//             }
//         }
//     }
// }

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }

#![no_std]
#![no_main]
pub mod drivers;
pub mod printk;
pub mod arch;
pub mod screen;
pub mod kspin_lock;
pub mod shell;
use core::panic::PanicInfo;
use core::fmt::Write;
use crate::drivers::keyboard;
use crate::screen::global::{init_screen_manager, screen_manager};
use crate::screen::screen::Writer;
use crate::shell::shell::{init_shell, global_shell};

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
                }
            }

            manager.switch_screen(0);
        }
    }
    
    keyboard::init_keyboard();
    init_shell();

    loop {
        // Poll keyboard for input
        if let Some(key_event) = keyboard::poll_keyboard() {
            match key_event {
                // chars - now handled by shell
                keyboard::KeyEvents::Character(c) => {
                    let mut shell = global_shell().lock();
                    shell.handle_character(c);
                }
                
                // special keys - some handled by shell, some by original handlers
                keyboard::KeyEvents::ArrowUp => {
                    keyboard::move_cursor_up();
                }
                keyboard::KeyEvents::ArrowDown => {
                    keyboard::move_cursor_down();
                }
                keyboard::KeyEvents::ArrowLeft => {
                    let mut shell = global_shell().lock();
                    shell.handle_arrow_left();
                }
                keyboard::KeyEvents::ArrowRight => {
                    let mut shell = global_shell().lock();
                    shell.handle_arrow_right();
                }
                keyboard::KeyEvents::Home => {
                    let mut shell = global_shell().lock();
                    shell.handle_home();
                }
                keyboard::KeyEvents::End => {
                    let mut shell = global_shell().lock();
                    shell.handle_end();
                }
                
                // editing keys - handled by shell
                keyboard::KeyEvents::BackSpace => {
                    let mut shell = global_shell().lock();
                    shell.handle_backspace();
                }
                keyboard::KeyEvents::Delete => {
                    let mut shell = global_shell().lock();
                    shell.handle_delete();
                }
                
                // enter key - handled by shell
                keyboard::KeyEvents::Enter => {
                    let mut shell = global_shell().lock();
                    shell.handle_enter();
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
                    } else {
                        // Re-display prompt on new screen
                        let mut shell = global_shell().lock();
                        shell.display_prompt();
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
                    } else {
                        // Re-display prompt on new screen
                        let mut shell = global_shell().lock();
                        shell.display_prompt();
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