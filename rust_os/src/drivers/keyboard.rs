// use crate::arch::x86::port::inb;
// use crate::printk;
// use crate::screen::global::screen_manager;
// use crate::screen::screen::{BUFFER_HEIGHT, BUFFER_WIDTH};

// // keyboard ports
// const KEYBOARD_DATA_PORT: u16 = 0x60;
// const KEYBOARD_STATUS_PORT: u16 = 0x64;
// const EXTENDED_KEY_PREFIX: u8 = 0xE0; // Prefix for extended keys (like arrow keys)

// // Convert scancode to ASCII
// const SCANCODE_TO_ASCII: [u8; 128] = [
//     0,  27, b'1', b'2', b'3', b'4', b'5', b'6',  // 0x00-0x07
//     b'7', b'8', b'9', b'0', b'-', b'=', 8,       // 0x08-0x0E (8 = backspace)
//     b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', // 0x0F-0x16
//     b'i', b'o', b'p', b'[', b']', b'\n',          // 0x17-0x1C (Enter)
//     0,    // 0x1D (Ctrl gauche)
//     b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', // 0x1E-0x26
//     b';', b'\'', b'`', 0,  // 0x27-0x2A (Shift gauche)
//     b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', // 0x2B-0x32
//     b',', b'.', b'/', 0,   // 0x33-0x36 (Shift droit)
//     b'*',
//     0,    // 0x38 (Alt gauche)
//     b' ', // 0x39 (Espace)
//     0,    // 0x3A (Caps lock)
//     // F1-F10 keys (0x3B-0x44)
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     0,    // 0x45 (Num lock)
//     0,    // 0x46 (Scroll lock)
//     b'7', b'8', b'9', b'-', // 0x47-0x4A (Keypad)
//     b'4', b'5', b'6', b'+', // 0x4B-0x4E (Keypad)
//     b'1', b'2', b'3',       // 0x4F-0x51 (Keypad)
//     b'0', b'.',             // 0x52-0x53 (Keypad)
//     // Reste rempli de zéros (0x54-0x7F = 44 éléments)
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
// ];

// #[derive(Clone, Copy, Debug)]
// pub enum KeyEvents {
//     Character(char),
//     ArrowUp,
//     ArrowDown,
//     ArrowLeft,
//     ArrowRight,
//     BackSpace,
//     Delete,
//     Enter,
//     Home,
//     End,
//     SwitchScreenLeft,
//     SwitchScreenRight
// }

// // Global state for modifier keys
// // These are unsafe because they are shared mutable state
// static mut SHIFT_PRESSED: bool = false;
// static mut CTRL_PRESSED: bool = false;
// static mut ALT_PRESSED: bool = false;
// static mut WAIT_FOR_EXTENDED: bool = false;

// pub fn init_keyboard() {
//     unsafe {
//         while keyboard_has_data() {
//             let _ = inb(KEYBOARD_DATA_PORT);
//         }
//         WAIT_FOR_EXTENDED = false;
//     }

//     printk!(LogLevel::Info, "Keyboard initialized.\n");
// }

// pub fn keyboard_has_data() -> bool {
//     unsafe {
//         inb(KEYBOARD_STATUS_PORT) & 0x01 != 0
//     }
// }

// pub fn poll_keyboard() -> Option<KeyEvents> {
//     if !keyboard_has_data() {
//         return None;
//     }

//     unsafe {
//         let scancode = inb(KEYBOARD_DATA_PORT);
//         handle_scancode(scancode)
//     }
// }

// pub fn handle_scancode(scancode: u8) -> Option<KeyEvents> {
//     unsafe {
//         if scancode == EXTENDED_KEY_PREFIX {
//             WAIT_FOR_EXTENDED = true;
//             return None; // Wait for the next scancode
//         }

//         let is_extended = WAIT_FOR_EXTENDED;
//         WAIT_FOR_EXTENDED = false;

//         let key_released = scancode & 0x80 != 0;
//         let key_code = scancode & 0x7F;

//         if is_extended && !key_released {
//             return match key_code {
//                 0x48 => Some(KeyEvents::ArrowUp),    // Up arrow
//                 0x50 => Some(KeyEvents::ArrowDown),  // Down arrow
//                 0x4B => {
//                     if CTRL_PRESSED {
//                         Some(KeyEvents::SwitchScreenLeft)  // Ctrl+Left = switch screen left
//                     } else {
//                         Some(KeyEvents::ArrowLeft)  // Left arrow
//                     }
//                 }
//                 0x4D => {
//                     if CTRL_PRESSED {
//                         Some(KeyEvents::SwitchScreenRight) // Ctrl+Right = switch screen right
//                     } else {
//                         Some(KeyEvents::ArrowRight) // Right arrow
//                     }
//                 }
//                 0x47 => Some(KeyEvents::Home),       // Home
//                 0x4F => Some(KeyEvents::End),        // End
//                 0x53 => Some(KeyEvents::Delete),     // Delete key (E0 53)
//                 _ => None,
//             };
//         }

//         match key_code {
//             0x2A | 0x36 => { // Shift keys
//                 SHIFT_PRESSED = !key_released;
//                 return None;
//             }
//             0x1D => { // Left Control key
//                 CTRL_PRESSED = !key_released;
//                 return None;
//             }
//             0x38 => { // Left Alt key
//                 ALT_PRESSED = !key_released;
//                 return None;
//             }
//             _ => {}
//         }

//         if key_released {
//             // If the key is released, we don't return any character
//             return None;
//         }

//         match key_code {
//             0x0E => return Some(KeyEvents::BackSpace), // Backspace
//             0x1C => return Some(KeyEvents::Enter),     // Enter
//             _ => {}
//         }

//         if (key_code as usize) < SCANCODE_TO_ASCII.len() {
//             let mut ascii = SCANCODE_TO_ASCII[key_code as usize];

//             if ascii == 0 {
//                 return None; // No mapping for this scancode
//             }

//             if SHIFT_PRESSED {
//                 // Converts to uppercase if Shift is pressed
//                 ascii = match ascii {
//                     b'a'..=b'z' => ascii - 32, // Convert to uppercase
//                     b'1' => '!' as u8,
//                     b'2' => '@' as u8,
//                     b'3' => '#' as u8,
//                     b'4' => '$' as u8,
//                     b'5' => '%' as u8,
//                     b'6' => '^' as u8,
//                     b'7' => '&' as u8,
//                     b'8' => '*' as u8,
//                     b'9' => '(' as u8,
//                     b'0' => ')' as u8,
//                     b'-' => '_' as u8,
//                     b'=' => '+' as u8,
//                     b'[' => '{' as u8,
//                     b']' => '}' as u8,
//                     b';' => ':' as u8,
//                     b'\'' => '"' as u8,
//                     b'\\' => '|' as u8,
//                     b',' => '<' as u8,
//                     b'.' => '>' as u8,
//                     b'/' => '?' as u8,
//                     _ => ascii,
//                 };
//             }
//             Some(KeyEvents::Character(ascii as char))
//         } else {
//             None // Invalid scancode
//         }
//     }
// }

// //=====================================================================================================================================
// //                                         CURSOR  MANAGEMENT - REFACTORED TO USE SCREEN MANAGER
// //=====================================================================================================================================

// pub fn move_cursor_up() {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         let row = active_screen.row_position;
//         if row > 0 {
//             active_screen.set_row_position(row - 1);
//         }
//     }
//     manager.update_cursor();
// }

// pub fn move_cursor_down() {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         let row = active_screen.row_position;
//         if row < BUFFER_HEIGHT - 1 {
//             active_screen.set_row_position(row + 1);
//         }
//     }
//     manager.update_cursor();
// }

// pub fn move_cursor_left() {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         let col = active_screen.column_position;
//         if col > 0 {
//             active_screen.set_column_position(col - 1);
//         }
//     }
//     manager.update_cursor();
// }

// pub fn move_cursor_right() {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         let col = active_screen.column_position;
//         if col < BUFFER_WIDTH - 1 {
//             active_screen.set_column_position(col + 1);
//         }
//     }
//     manager.update_cursor();
// }

// pub fn write_at_cursor(c: char) {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         active_screen.write_byte(c as u8);
//     }
//     manager.flush_to_physical();
//     manager.update_cursor();
// }

// pub fn move_cursor_home() {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         active_screen.set_cursor_position(0, 0);
//     }
//     manager.update_cursor();
// }

// pub fn move_cursor_end() {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         active_screen.set_cursor_position(BUFFER_HEIGHT - 1, BUFFER_WIDTH - 1);
//     }
//     manager.update_cursor();
// }

// pub fn handle_backspace() {
//     let mut manager = screen_manager().lock();
//     let active_screen_id = manager.active_screen_id;
//     if let Some(active_screen) = &mut manager.screens[active_screen_id] {
//         let col = active_screen.column_position;
//         if col > 0 {
//             active_screen.set_column_position(col - 1);
//             active_screen.write_byte_at(
//                 active_screen.row_position,
//                 active_screen.column_position,
//                 b' '
//             );
//         }
//     }
//     manager.flush_to_physical();
//     manager.update_cursor();
// }

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

use crate::arch::x86::port::inb;
use crate::printk;
use crate::screen::global::screen_manager;
use crate::screen::screen::{BUFFER_HEIGHT, BUFFER_WIDTH};

// keyboard ports
const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;
const EXTENDED_KEY_PREFIX: u8 = 0xE0; // Prefix for extended keys (like arrow keys)

// Convert scancode to ASCII
const SCANCODE_TO_ASCII: [u8; 128] = [
    0,  27, b'1', b'2', b'3', b'4', b'5', b'6',  // 0x00-0x07
    b'7', b'8', b'9', b'0', b'-', b'=', 8,       // 0x08-0x0E (8 = backspace)
    b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', // 0x0F-0x16
    b'i', b'o', b'p', b'[', b']', b'\n',          // 0x17-0x1C (Enter)
    0,    // 0x1D (Ctrl gauche)
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', // 0x1E-0x26
    b';', b'\'', b'`', 0,  // 0x27-0x2A (Shift gauche)
    b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', // 0x2B-0x32
    b',', b'.', b'/', 0,   // 0x33-0x36 (Shift droit)
    b'*',
    0,    // 0x38 (Alt gauche)
    b' ', // 0x39 (Espace)
    0,    // 0x3A (Caps lock)
    // F1-F10 keys (0x3B-0x44)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,    // 0x45 (Num lock)
    0,    // 0x46 (Scroll lock)
    b'7', b'8', b'9', b'-', // 0x47-0x4A (Keypad)
    b'4', b'5', b'6', b'+', // 0x4B-0x4E (Keypad)
    b'1', b'2', b'3',       // 0x4F-0x51 (Keypad)
    b'0', b'.',             // 0x52-0x53 (Keypad)
    // Reste rempli de zéros (0x54-0x7F = 44 éléments)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

#[derive(Clone, Copy, Debug)]
pub enum KeyEvents {
    Character(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    BackSpace,
    Delete,
    Enter,
    Home,
    End,
    SwitchScreenLeft,
    SwitchScreenRight
}

// Global state for modifier keys
// These are unsafe because they are shared mutable state
static mut SHIFT_PRESSED: bool = false;
static mut CTRL_PRESSED: bool = false;
static mut ALT_PRESSED: bool = false;
static mut WAIT_FOR_EXTENDED: bool = false;

pub fn init_keyboard() {
    unsafe {
        while keyboard_has_data() {
            let _ = inb(KEYBOARD_DATA_PORT);
        }
        WAIT_FOR_EXTENDED = false;
    }

    printk!(LogLevel::Info, "Keyboard initialized.\n");
}

pub fn keyboard_has_data() -> bool {
    unsafe {
        inb(KEYBOARD_STATUS_PORT) & 0x01 != 0
    }
}

pub fn poll_keyboard() -> Option<KeyEvents> {
    if !keyboard_has_data() {
        return None;
    }

    unsafe {
        let scancode = inb(KEYBOARD_DATA_PORT);
        handle_scancode(scancode)
    }
}

pub fn handle_scancode(scancode: u8) -> Option<KeyEvents> {
    unsafe {
        if scancode == EXTENDED_KEY_PREFIX {
            WAIT_FOR_EXTENDED = true;
            return None; // Wait for the next scancode
        }

        let is_extended = WAIT_FOR_EXTENDED;
        WAIT_FOR_EXTENDED = false;

        let key_released = scancode & 0x80 != 0;
        let key_code = scancode & 0x7F;

        if is_extended && !key_released {
            return match key_code {
                0x48 => Some(KeyEvents::ArrowUp),    // Up arrow
                0x50 => Some(KeyEvents::ArrowDown),  // Down arrow
                0x4B => {
                    if CTRL_PRESSED {
                        Some(KeyEvents::SwitchScreenLeft)  // Ctrl+Left = switch screen left
                    } else {
                        Some(KeyEvents::ArrowLeft)  // Left arrow
                    }
                }
                0x4D => {
                    if CTRL_PRESSED {
                        Some(KeyEvents::SwitchScreenRight) // Ctrl+Right = switch screen right
                    } else {
                        Some(KeyEvents::ArrowRight) // Right arrow
                    }
                }
                0x47 => Some(KeyEvents::Home),       // Home
                0x4F => Some(KeyEvents::End),        // End
                0x53 => Some(KeyEvents::Delete),     // Delete key (E0 53)
                _ => None,
            };
        }

        match key_code {
            0x2A | 0x36 => { // Shift keys
                SHIFT_PRESSED = !key_released;
                return None;
            }
            0x1D => { // Left Control key
                CTRL_PRESSED = !key_released;
                return None;
            }
            0x38 => { // Left Alt key
                ALT_PRESSED = !key_released;
                return None;
            }
            _ => {}
        }

        if key_released {
            // If the key is released, we don't return any character
            return None;
        }

        match key_code {
            0x0E => return Some(KeyEvents::BackSpace), // Backspace
            0x1C => return Some(KeyEvents::Enter),     // Enter
            _ => {}
        }

        if (key_code as usize) < SCANCODE_TO_ASCII.len() {
            let mut ascii = SCANCODE_TO_ASCII[key_code as usize];

            if ascii == 0 {
                return None; // No mapping for this scancode
            }

            if SHIFT_PRESSED {
                // Converts to uppercase if Shift is pressed
                ascii = match ascii {
                    b'a'..=b'z' => ascii - 32, // Convert to uppercase
                    b'1' => '!' as u8,
                    b'2' => '@' as u8,
                    b'3' => '#' as u8,
                    b'4' => '$' as u8,
                    b'5' => '%' as u8,
                    b'6' => '^' as u8,
                    b'7' => '&' as u8,
                    b'8' => '*' as u8,
                    b'9' => '(' as u8,
                    b'0' => ')' as u8,
                    b'-' => '_' as u8,
                    b'=' => '+' as u8,
                    b'[' => '{' as u8,
                    b']' => '}' as u8,
                    b';' => ':' as u8,
                    b'\'' => '"' as u8,
                    b'\\' => '|' as u8,
                    b',' => '<' as u8,
                    b'.' => '>' as u8,
                    b'/' => '?' as u8,
                    _ => ascii,
                };
            }
            Some(KeyEvents::Character(ascii as char))
        } else {
            None // Invalid scancode
        }
    }
}

//=====================================================================================================================================
//                                         CURSOR  MANAGEMENT - LEGACY FUNCTIONS PRESERVED FOR COMPATIBILITY
//=====================================================================================================================================

// These functions are kept for backward compatibility and special cases
// Most cursor management is now handled by the shell

pub fn move_cursor_up() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        let row = active_screen.row_position;
        if row > 0 {
            active_screen.set_row_position(row - 1);
        }
    }
    manager.update_cursor();
}

pub fn move_cursor_down() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        let row = active_screen.row_position;
        if row < BUFFER_HEIGHT - 1 {
            active_screen.set_row_position(row + 1);
        }
    }
    manager.update_cursor();
}

// Legacy functions preserved for special use cases
pub fn move_cursor_left() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        let col = active_screen.column_position;
        if col > 0 {
            active_screen.set_column_position(col - 1);
        }
    }
    manager.update_cursor();
}

pub fn move_cursor_right() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        let col = active_screen.column_position;
        if col < BUFFER_WIDTH - 1 {
            active_screen.set_column_position(col + 1);
        }
    }
    manager.update_cursor();
}

pub fn write_at_cursor(c: char) {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        active_screen.write_byte(c as u8);
    }
    manager.flush_to_physical();
    manager.update_cursor();
}

pub fn move_cursor_home() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        active_screen.set_cursor_position(0, 0);
    }
    manager.update_cursor();
}

pub fn move_cursor_end() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        active_screen.set_cursor_position(BUFFER_HEIGHT - 1, BUFFER_WIDTH - 1);
    }
    manager.update_cursor();
}

pub fn handle_backspace() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        let col = active_screen.column_position;
        if col > 0 {
            active_screen.set_column_position(col - 1);
            active_screen.write_byte_at(
                active_screen.row_position,
                active_screen.column_position,
                b' '
            );
        }
    }
    manager.flush_to_physical();
    manager.update_cursor();
}

pub fn handle_delete() {
    let mut manager = screen_manager().lock();
    let active_screen_id = manager.active_screen_id;
    if let Some(active_screen) = &mut manager.screens[active_screen_id] {
        active_screen.write_byte_at(
            active_screen.row_position,
            active_screen.column_position,
            b' '
        );
    }
    manager.flush_to_physical();
    manager.update_cursor();
}