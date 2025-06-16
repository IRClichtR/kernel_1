// use crate::arch::x86::port::{inb, outb};
// use crate::{printk, printk::printk::{LogLevel}};

// // CONST FOR KEYBOARD CONTROLLER
// pub const KEYBOARD_DATA_PORT: u16 = 0x60;
// pub const KEYBOARD_STATUS_PORT: u16 = 0x64;
// pub const KEYBOARD_COMMAND_PORT: u16 = 0x64;

// // Keyboard command codes
// const KC_READ_CONFIG: u8 = 0x20;
// const KC_WRITE_CONFIG: u8 = 0x60;

// // Buffer for scan codes
// const BUFFER_SIZE: usize = 16;
// static mut KEYBOARD_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
// static mut BUFFER_HEAD: usize = 0;
// static mut BUFFER_TAIL: usize = 0;
// static mut KEY_AVAILABLE: bool = false;

// #[inline(never)]
// pub unsafe fn initialize_keyboard() {
//     // Beep the PC speaker to confirm function entry
//     outb(0x43, 0xB6);
//     outb(0x42, 0x1B);
//     outb(0x42, 0x00);
//     let tmp = inb(0x61);
//     outb(0x61, tmp | 3);

//     // Wait for the keyboard controller to be ready
//     printk!(LogLevel::Debug, "Keyboard controller is ready\n");
//     while (inb(KEYBOARD_STATUS_PORT) & 2) != 0 {}

//     // Read the current configuration byte
//     outb(KEYBOARD_COMMAND_PORT, KC_READ_CONFIG);

//     // wAIT FOR DATA TO BE READY
//     while (inb(KEYBOARD_STATUS_PORT) & 1) == 0 {}

//     //Read config byte
//     let config_byte = inb(KEYBOARD_DATA_PORT);

//     let new_config_byte = (config_byte | 1) & 0xFD; // Disable the keyboard interrupt 

//     // Wait for the keyboard controller to be ready
//     while (inb(KEYBOARD_STATUS_PORT) & 2) != 0 {}

//     // Write the new configuration byte
//     outb(KEYBOARD_COMMAND_PORT, KC_WRITE_CONFIG);

//     // Wait for the keyboard controller to be ready
//     while (inb(KEYBOARD_STATUS_PORT) & 2) != 0 {}
//     outb(KEYBOARD_DATA_PORT, new_config_byte);

//     while (inb(KEYBOARD_STATUS_PORT) & 1) != 0 {
//         inb(KEYBOARD_DATA_PORT); // Read and discard any data
//     };

//     // Turn off the speaker when done
//     let tmp = inb(0x61);
//     outb(0x61, tmp & 0xFC);
// }

// pub fn handle_scan_code(scan_code: u8) {
//     unsafe {
//         // Simple mapping for common keys (add more as needed)
//         let character = match scan_code {
//             0x1E => 'a',
//             0x30 => 'b',
//             0x2E => 'c',
//             0x20 => 'd',
//             0x12 => 'e',
//             0x21 => 'f',
//             0x22 => 'g',
//             0x23 => 'h',
//             0x17 => 'i',
//             0x24 => 'j',
//             0x25 => 'k',
//             0x26 => 'l',
//             0x32 => 'm',
//             0x31 => 'n',
//             0x18 => 'o',
//             0x19 => 'p',
//             0x10 => 'q',
//             0x13 => 'r',
//             0x1F => 's',
//             0x14 => 't',
//             0x16 => 'u',
//             0x2F => 'v',
//             0x11 => 'w',
//             0x2D => 'x',
//             0x15 => 'y',
//             0x2C => 'z',
//             0x39 => ' ', // Space
//             0x1C => '\n', // Enter
//             _ => '\0'    // Null character for unmapped keys
//         };
        
//         // Only print if it's a valid character (not null)
//         if character != '\0' {
//             // Print both the scan code and the character
//             printk!(LogLevel::Info, 
//                 "Key: '{}' (scan code: {:#x})\n", character, scan_code);
//         } else {
//             // Just print the scan code for unmapped keys
//             printk!(LogLevel::Debug, 
//                 "Unmapped scan code: {:#x}\n", scan_code);
//         }
//         // printk!(LogLevel::Info, "Scan code: {:#X}\n", scan_code);
//     }
// }

// // Add a scan code to the buffer (called from interrupt handler)
// pub unsafe fn add_scan_code(scan_code: u8) {
//     let next_head = (BUFFER_HEAD + 1) % KEYBOARD_BUFFER.len();
//     if next_head != BUFFER_TAIL {
//         KEYBOARD_BUFFER[BUFFER_HEAD] = scan_code;
//         BUFFER_HEAD = next_head;
//         KEY_AVAILABLE = true;
//     }
// }

// pub unsafe fn process_keyboard() -> bool {
//     if !KEY_AVAILABLE {
//         return false;
//     }

//     let mut processed = false;

//     while BUFFER_HEAD != BUFFER_TAIL {
//         let scan_code = KEYBOARD_BUFFER[BUFFER_TAIL];
//         BUFFER_TAIL = (BUFFER_TAIL + 1) % BUFFER_SIZE;

//         handle_scan_code(scan_code);
//         processed = true;
//     }

//     if BUFFER_HEAD == BUFFER_TAIL {
//         KEY_AVAILABLE = false;
//     }

//     processed
// }

// // pub fn is_key_available() -> bool {
// //     unsafe {
// //         BUFFER_HEAD != BUFFER_TAIL
// //     }
// // }

// // pub fn get_key() -> Option<u8> {
// //     unsafe {
// //         if BUFFER_HEAD != BUFFER_TAIL {
// //             let key = KEYBOARD_BUFFER[BUFFER_TAIL];
// //             BUFFER_TAIL = (BUFFER_TAIL + 1) % BUFFER_SIZE;
// //             Some(key)
// //         } else {
// //             None
// //         }
// //     }
// // }