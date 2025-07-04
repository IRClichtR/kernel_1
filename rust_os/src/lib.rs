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

// src/lib.rs
// Updated kernel library with integrated shell system

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod printk;
pub mod screen;
pub mod arch;
pub mod drivers;
pub mod shell;
pub mod kspin_lock;

use core::panic::PanicInfo;
use screen::SCREEN_MANAGER;

/// Entry point for the kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Initializing kernel...");
    
    // Initialize screen manager
    unsafe {
        SCREEN_MANAGER.lock().init();
    }
    
    // Initialize shell system
    shell::init_shell();
    
    println!("Kernel initialized successfully!");
    
    // Main kernel loop
    loop {
        // Process shell input/output
        shell::process_shell();
        
        // Prevent tight loop - add small delay
        for _ in 0..1000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}

/// Panic handler for kernel
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// Test panic handler
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[failed]\n");
    println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

/// Exit codes for QEMU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exit QEMU with the given exit code
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use arch::x86::port::Port;
    
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
    
    loop {}
}

/// Test runner for kernel tests
#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    
    for test in tests {
        test.run();
    }
    
    exit_qemu(QemuExitCode::Success);
}

/// Trait for testable functions
pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

/// Test main function
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

/// Kernel initialization function (for modular testing)
pub fn init_kernel() {
    // Initialize screen manager
    unsafe {
        SCREEN_MANAGER.lock().init();
    }
    
    // Initialize shell system
    shell::init_shell();
    
    println!("Kernel components initialized");
}

/// Main kernel loop function (for modular testing)
pub fn kernel_main_loop() -> ! {
    loop {
        // Process shell input/output
        shell::process_shell();
        
        // Prevent tight loop - add small delay
        for _ in 0..1000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}

/// Public API for kernel operations
pub mod api {
    use super::*;
    
    /// Execute a shell command
    pub fn execute_shell_command(command: &str) -> shell::CommandResult {
        shell::execute_command(command)
    }
    
    /// Write to kernel output
    pub fn kernel_write(text: &str) {
        shell::shell_write(text);
    }
    
    /// Clear kernel screen
    pub fn kernel_clear_screen() {
        shell::shell_clear();
    }
    
    /// Get kernel status
    pub fn get_kernel_status() -> KernelStatus {
        KernelStatus {
            shell_state: shell::get_shell_state(),
            has_input: shell::shell_has_input(),
        }
    }
}

/// Kernel status structure
#[derive(Debug, Clone, Copy)]
pub struct KernelStatus {
    pub shell_state: shell::ShellState,
    pub has_input: bool,
}

/// Integration tests for the shell system
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_kernel_init() {
        init_kernel();
        // Test passes if no panic occurs
    }
    
    #[test_case]
    fn test_shell_command_execution() {
        init_kernel();
        let result = api::execute_shell_command("help");
        assert_eq!(result, shell::CommandResult::Success);
    }
    
    #[test_case]
    fn test_shell_invalid_command() {
        init_kernel();
        let result = api::execute_shell_command("invalid_command");
        assert_eq!(result, shell::CommandResult::InvalidCommand);
    }
    
    #[test_case]
    fn test_shell_state_machine() {
        init_kernel();
        let state = shell::get_shell_state();
        // State should be properly initialized
        assert!(matches!(state, shell::ShellState::Waiting | shell::ShellState::Uninitialized));
    }
}