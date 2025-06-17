use crate::arch::x86::port::{inb, outb};
use crate::printk;
use crate::printk::printk::LogLevel;

// keyboard ports
const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;

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

static mut SHIFT_PRESSED: bool = false;
static mut CTRL_PRESSED: bool = false;
static mut ALT_PRESSED: bool = false;

pub fn init_keyboard() {
    unsafe {
        while keyboard_has_data() {
            let _ = inb(KEYBOARD_DATA_PORT);
        }
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
   let key_released = scancode & 0x80 != 0;
   let key_code = scancode & 0x7F;

   unsafe {
        match key_code {
            0x2A | 0x36 => { // shift keys
                SHIFT_PRESSED = !key_released;
                return None;
            }
            0x1D => { // left control key
                CTRL_PRESSED = !key_released;
                return None;
            }
            _ => {} // other keys
        }
   }

   // return non on modificators
   if matches!(key_code, 0x2A | 0x36 | 0x1D) {
        return None; // Ignore modifier keys
    }

   if key_released {
        return None;
    }

    if (key_code as usize) < SCANCODE_TO_ASCII.len() {
        let mut ascii = SCANCODE_TO_ASCII[key_code as usize];

        if ascii == 0 {
            return None; // No mapping for this scancode
        }

        unsafe {
            if SHIFT_PRESSED {
                //Conferts to uppercase if Shift is pressed
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
        }

        Some(ascii as char)
    } else {
        None // Invalid scancode
    }
}