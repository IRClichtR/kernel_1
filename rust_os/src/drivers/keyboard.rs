use crate::arch::x86::port::inb;
use crate::printk;
use crate::printk::printk::LogLevel;
use crate::vga_buffer::vga_buffer::WRITER;

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
    Enter,
    Home,
    End
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
            let _ = inb(KEYBOARD_DATA_PORT); // Clear any existing data in the buffer
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

pub fn poll_keyboard() -> Option<char> {
    if !keyboard_has_data() {
        return None;
    }

    unsafe {
        let scancode = inb(KEYBOARD_DATA_PORT);
        handle_scancode(scancode)
    }
}

pub fn handle_scancode(scancode: u8) -> Option<char> {
    unsafe {
        if scancode == EXTENDED_KEY_PREFIX {
            WAIT_FOR_EXTENDED = true;
            return None; // Wait for the next scancode
        }

        let is_extended = WAIT_FOR_EXTENDED;
        WAIT_FOR_EXTENDED = false;

        let key_released = scancode & 0x80 != 0;
        let key_code = scancode & 0x7F;

        if key_code == scancode & 0x80 {
            // Handle extended keys (like arrow keys)
            return match key_code {
                0x48 => Some(KeyEvents::ArrowUp),    // Up arrow
                0x50 => Some(KeyEvents::ArrowDown),  // Down arrow
                0x4B => Some(KeyEvents::ArrowLeft),  // Left arrow
                0x4D => Some(KeyEvents::ArrowRight), // Right arrow
                0x47 => Some(KeyEvents::Home),       // Home
                0x4F => Some(KeyEvents::End),        // End
                _ => None,
            };
        }

        if match key_code {
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
            _ => false, // Other keys
        } {
            return None; // Ignore modifier keys
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
            Some(ascii as char)            
        } else {
            None // Invalid scancode
        }
    }
}

//=====================================================================================================================================
//                                         CURSOR  MANAGEMENT
//=====================================================================================================================================

pub fn move_cursor_up() {
    WRITER.lock();
    let row = WRITER.get_row();
    if row > 0 {
        WRITER.set_row(row - 1);
    }
}

pub fn move_cursor_down() {
    WRITER.lock();
    let row = WRITER.get_row();
    if row < WRITER.get_buffer().height - 1 {
        WRITER.set_row(row + 1);
    }
}

pub fn move_cursor_left() {
    WRITER.lock();
    let col = WRITER.get_col();
    if col > 0 {
        WRITER.set_col(col - 1);
    }
}

pub fn move_cursor_right() {
    WRITER.lock();
    let col = WRITER.get_col();
    if col < WRITER.get_buffer().width - 1 {
        WRITER.set_col(col + 1);
    }
}

pub fn write_at_cursor(c: char) {
    WRITER.lock();
    WRITER.write_byte(c as u8);
}

pub fn move_cursor_home() {
    WRITER.lock();
    WRITER.set_cursor_position(0, 0);
}

pub fn move_cursor_end() {
    WRITER.lock();
    let last_row = WRITER.get_buffer().height - 1;
    let last_col = WRITER.get_buffer().width - 1;
    WRITER.set_cursor_position(last_row, last_col);
}

pub fn handle_backspace() {
    WRITER.lock();
    move_cursor_left();
    WRITER.write_byte(b' '); // Write a space to clear the character
    move_cursor_left(); // Move cursor back to the left after writing space
}