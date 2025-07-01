// // #![no_std]
// // #![no_main]
// // pub mod drivers;
// // pub mod printk;
// // pub mod arch;
// // pub mod screen;
// // pub mod kspin_lock;
// // use core::panic::PanicInfo;
// // use core::fmt::Write;
// // use crate::drivers::keyboard;
// // use crate::screen::global::{init_screen_manager, screen_manager};
// // use crate::screen::screen::Writer;

// // #[no_mangle]
// // pub extern "C" fn kernel_main() -> ! {
// //     init_screen_manager();
    
// //     {
// //         let mut manager = screen_manager().lock();
    
// //         if let Some(_screen_id) = manager.create_screen() {            
// //             if manager.switch_screen(1) {
// //                 if let Some(screen) = &mut manager.screens[1] {
// //                     let mut writer = Writer::new(screen);
// //                     write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
// //                     write!(writer, "\n").unwrap();
// //                 }
// //             }

// //             manager.switch_screen(0);
// //         }
// //     }
    
// //     keyboard::init_keyboard();

// //     loop {
// //         // Poll keyboard for input
// //         if let Some(key_event) = keyboard::poll_keyboard() {
// //             match key_event {
// //                 // chars - now uses screen manager
// //                 keyboard::KeyEvents::Character(c) => {
// //                     let mut manager = screen_manager().lock();
// //                     let active_screen_id = manager.active_screen_id; // Store the ID first
// //                     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
// //                         use crate::screen::screen::Writer;
// //                         let mut writer = Writer::new(active_screen);
// //                         writer.write_byte(c as u8);
// //                     }
// //                     manager.flush_to_physical();
// //                     manager.update_cursor();
// //                 }
                
// //                 // special keys
// //                 keyboard::KeyEvents::ArrowUp => {
// //                     keyboard::move_cursor_up();
// //                 }
// //                 keyboard::KeyEvents::ArrowDown => {
// //                     keyboard::move_cursor_down();
// //                 }
// //                 keyboard::KeyEvents::ArrowLeft => {
// //                     keyboard::move_cursor_left();
// //                 }
// //                 keyboard::KeyEvents::ArrowRight => {
// //                     keyboard::move_cursor_right();
// //                 }
// //                 keyboard::KeyEvents::Home => {
// //                     keyboard::move_cursor_home();
// //                 }
// //                 keyboard::KeyEvents::End => {
// //                     keyboard::move_cursor_end();
// //                 }
                
// //                 // editing keys
// //                 keyboard::KeyEvents::BackSpace => {
// //                     keyboard::handle_backspace();
// //                 }
// //                 keyboard::KeyEvents::Delete => {
// //                     keyboard::handle_delete();
// //                 }
                
// //                 // enter key - now uses screen manager
// //                 keyboard::KeyEvents::Enter => {
// //                     let mut manager = screen_manager().lock();
// //                     let active_screen_id = manager.active_screen_id; // Store the ID first
// //                     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
// //                         use crate::screen::screen::Writer;
// //                         let mut writer = Writer::new(active_screen);
// //                         writer.write_byte(b'\n');  // This will trigger new_line logic in Writer
// //                     }
// //                     manager.flush_to_physical();
// //                     manager.update_cursor();
// //                 }
// //                 keyboard::KeyEvents::SwitchScreenLeft => {
// //                     let switch_successful = {
// //                         let mut manager = screen_manager().lock();
// //                         let current_screen = manager.active_screen_id;
// //                         let new_screen = if current_screen == 0 { 1 } else { 0 };
// //                         manager.switch_screen(new_screen)
// //                     };
                    
// //                     if !switch_successful {
// //                         printk!(LogLevel::Critical, "Fatal error switching the screen\n");
// //                     }
// //                 }
// //                 keyboard::KeyEvents::SwitchScreenRight => {
// //                     let switch_successful = {
// //                         let mut manager = screen_manager().lock();
// //                         let current_screen = manager.active_screen_id;
// //                         let new_screen = if current_screen == 0 { 1 } else { 0 };
// //                         manager.switch_screen(new_screen)
// //                     };
                    
// //                     if !switch_successful {
// //                         printk!(LogLevel::Critical, "Fatal error switching the screen\n");
// //                     }
// //                 }
// //             }
// //         }
// //     }
// // }

// // #[panic_handler]
// // fn panic(_info: &PanicInfo) -> ! {
// //     loop {}
// // }

// #![no_std]
// #![no_main]
// pub mod drivers;
// pub mod printk;
// pub mod arch;
// pub mod screen;
// pub mod kspin_lock;
// pub mod shell;
// use core::panic::PanicInfo;
// use core::fmt::Write;
// use crate::drivers::keyboard;
// use crate::screen::global::{init_screen_manager, screen_manager};
// use crate::screen::screen::Writer;
// use crate::shell::shell::{init_shell, global_shell};

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
//     init_shell();

//     loop {
//         // Poll keyboard for input
//         if let Some(key_event) = keyboard::poll_keyboard() {
//             match key_event {
//                 // chars - now handled by shell
//                 keyboard::KeyEvents::Character(c) => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_character(c);
//                 }
                
//                 // special keys - some handled by shell, some by original handlers
//                 keyboard::KeyEvents::ArrowUp => {
//                     keyboard::move_cursor_up();
//                 }
//                 keyboard::KeyEvents::ArrowDown => {
//                     keyboard::move_cursor_down();
//                 }
//                 keyboard::KeyEvents::ArrowLeft => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_arrow_left();
//                 }
//                 keyboard::KeyEvents::ArrowRight => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_arrow_right();
//                 }
//                 keyboard::KeyEvents::Home => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_home();
//                 }
//                 keyboard::KeyEvents::End => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_end();
//                 }
                
//                 // editing keys - handled by shell
//                 keyboard::KeyEvents::BackSpace => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_backspace();
//                 }
//                 keyboard::KeyEvents::Delete => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_delete();
//                 }
                
//                 // enter key - handled by shell
//                 keyboard::KeyEvents::Enter => {
//                     let mut shell = global_shell().lock();
//                     shell.handle_enter();
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
//                     } else {
//                         // Re-display prompt on new screen
//                         let mut shell = global_shell().lock();
//                         shell.display_prompt();
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
//                     } else {
//                         // Re-display prompt on new screen
//                         let mut shell = global_shell().lock();
//                         shell.display_prompt();
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
//use crate::printk::{printk, LogLevel};

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Initialize screen manager first
    init_screen_manager();
    
    printk!(LogLevel::Info, "Kernel: Screen manager initialized\n");
    
    {
        let mut manager = screen_manager().lock();
    
        if let Some(_screen_id) = manager.create_screen() {            
            printk!(LogLevel::Info, "Kernel: Created screen 1\n");
            
            if manager.switch_screen(1) {
                printk!(LogLevel::Info, "Kernel: Switched to screen 1\n");
                
                if let Some(screen) = &mut manager.screens[1] {
                    let mut writer = Writer::new(screen);
                    write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
                    write!(writer, "\n").unwrap();
                }
            }

            // Switch back to screen 0 for kernel messages
            manager.switch_screen(0);
            printk!(LogLevel::Info, "Kernel: Switched back to screen 0\n");
        }
    }
    
    // Initialize keyboard
    keyboard::init_keyboard();
    printk!(LogLevel::Info, "Kernel: Keyboard initialized\n");
    
    // Initialize shell
    init_shell();
    printk!(LogLevel::Info, "Kernel: Shell initialized\n");
    
    // Switch to user terminal for shell
    {
        let mut manager = screen_manager().lock();
        manager.switch_screen(1);
        printk!(LogLevel::Info, "Kernel: Switched to user terminal for shell\n");
    }

    printk!(LogLevel::Info, "Kernel: Entering main loop\n");

    loop {
        // Poll keyboard for input
        if let Some(key_event) = keyboard::poll_keyboard() {
            printk!(LogLevel::Debug, "Kernel: Received key event\n");
            
            match key_event {
                // chars - now handled by shell
                keyboard::KeyEvents::Character(c) => {
                    printk!(LogLevel::Debug, "Kernel: Character '{}' -> Shell\n", c);
                    let mut shell = global_shell().lock();
                    shell.handle_character(c);
                }
                
                // special keys - some handled by shell, some by original handlers
                keyboard::KeyEvents::ArrowUp => {
                    printk!(LogLevel::Debug, "Kernel: Arrow Up -> Keyboard handler\n");
                    keyboard::move_cursor_up();
                }
                keyboard::KeyEvents::ArrowDown => {
                    printk!(LogLevel::Debug, "Kernel: Arrow Down -> Keyboard handler\n");
                    keyboard::move_cursor_down();
                }
                keyboard::KeyEvents::ArrowLeft => {
                    printk!(LogLevel::Debug, "Kernel: Arrow Left -> Shell\n");
                    let mut shell = global_shell().lock();
                    shell.handle_arrow_left();
                }
                keyboard::KeyEvents::ArrowRight => {
                    printk!(LogLevel::Debug, "Kernel: Arrow Right -> Shell\n");
                    let mut shell = global_shell().lock();
                    shell.handle_arrow_right();
                }
                keyboard::KeyEvents::Home => {
                    printk!(LogLevel::Debug, "Kernel: Home -> Shell\n");
                    let mut shell = global_shell().lock();
                    shell.handle_home();
                }
                keyboard::KeyEvents::End => {
                    printk!(LogLevel::Debug, "Kernel: End -> Shell\n");
                    let mut shell = global_shell().lock();
                    shell.handle_end();
                }
                
                // editing keys - handled by shell
                keyboard::KeyEvents::BackSpace => {
                    printk!(LogLevel::Debug, "Kernel: Backspace -> Shell\n");
                    let mut shell = global_shell().lock();
                    shell.handle_backspace();
                }
                keyboard::KeyEvents::Delete => {
                    printk!(LogLevel::Debug, "Kernel: Delete -> Shell\n");
                    let mut shell = global_shell().lock();
                    shell.handle_delete();
                }
                
                // enter key - handled by shell
                keyboard::KeyEvents::Enter => {
                    printk!(LogLevel::Debug, "Kernel: Enter -> Shell\n");
                    let mut shell = global_shell().lock();
                    shell.handle_enter();
                }
                keyboard::KeyEvents::SwitchScreenLeft => {
                    printk!(LogLevel::Debug, "Kernel: Switch Screen Left\n");
                    let switch_successful = {
                        let mut manager = screen_manager().lock();
                        let current_screen = manager.active_screen_id;
                        let new_screen = if current_screen == 0 { 1 } else { 0 };
                        printk!(LogLevel::Info, "Kernel: Switching from screen {} to screen {}\n", current_screen, new_screen);
                        manager.switch_screen(new_screen)
                    };
                    
                    if !switch_successful {
                        printk!(LogLevel::Critical, "Fatal error switching the screen\n");
                    } else {
                        // Re-display prompt on new screen if we switched to user terminal
                        let manager = screen_manager().lock();
                        if manager.active_screen_id == 1 {
                            drop(manager); // Release the lock before taking shell lock
                            let mut shell = global_shell().lock();
                            shell.display_prompt();
                        }
                    }
                }
                keyboard::KeyEvents::SwitchScreenRight => {
                    printk!(LogLevel::Debug, "Kernel: Switch Screen Right\n");
                    let switch_successful = {
                        let mut manager = screen_manager().lock();
                        let current_screen = manager.active_screen_id;
                        let new_screen = if current_screen == 0 { 1 } else { 0 };
                        printk!(LogLevel::Info, "Kernel: Switching from screen {} to screen {}\n", current_screen, new_screen);
                        manager.switch_screen(new_screen)
                    };
                    
                    if !switch_successful {
                        printk!(LogLevel::Critical, "Fatal error switching the screen\n");
                    } else {
                        // Re-display prompt on new screen if we switched to user terminal
                        let manager = screen_manager().lock();
                        if manager.active_screen_id == 1 {
                            drop(manager); // Release the lock before taking shell lock
                            let mut shell = global_shell().lock();
                            shell.display_prompt();
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