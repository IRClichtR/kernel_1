use crate::arch::x86::port::inb;
use crate::printk;
use crate::screen::global::screen_manager;
use crate::screen::screen::{BUFFER_HEIGHT, BUFFER_WIDTH};

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;
const EXTENDED_KEY_PREFIX: u8 = 0xE0;

const SCANCODE_TO_ASCII: [u8; 128] = [
    0,  27, b'1', b'2', b'3', b'4', b'5', b'6',
    b'7', b'8', b'9', b'0', b'-', b'=', 8,
    b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u',
    b'i', b'o', b'p', b'[', b']', b'\n',
    0,
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l',
    b';', b'\'', b'`', 0,
    b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm',
    b',', b'.', b'/', 0,
    b'*',
    0,
    b' ',
    0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
    0,
    b'7', b'8', b'9', b'-',
    b'4', b'5', b'6', b'+',
    b'1', b'2', b'3',
    b'0', b'.',
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
        SHIFT_PRESSED = false;
        CTRL_PRESSED = false;
        ALT_PRESSED = false;
    }

    printk!(LogLevel::Info, "Keyboard initialized.\n");
}

pub fn keyboard_has_data() -> bool {
    unsafe {
        inb(KEYBOARD_STATUS_PORT) & 0x01 != 0
    }
}

pub fn reset_keyboard_state() {
    unsafe {
        WAIT_FOR_EXTENDED = false;
        SHIFT_PRESSED = false;
        CTRL_PRESSED = false;
        ALT_PRESSED = false;
    }
}

pub fn poll_keyboard() -> Option<KeyEvents> {
    if !keyboard_has_data() {
        return None;
    }

    unsafe {
        let scancode = inb(KEYBOARD_DATA_PORT);
        
        if scancode == 0xFF {
            return None;
        }
        
        if scancode == EXTENDED_KEY_PREFIX {
            WAIT_FOR_EXTENDED = true;
            return None;
        }
        
        let is_extended = WAIT_FOR_EXTENDED;
        WAIT_FOR_EXTENDED = false;
        
        let key_released = (scancode & 0x80) != 0;
        let key_code = scancode & 0x7F;
        
        if key_released {
            match key_code {
                0x1D => CTRL_PRESSED = false,
                0x2A | 0x36 => SHIFT_PRESSED = false,
                0x38 => ALT_PRESSED = false,
                _ => {}
            }
            return None;
        }
        
        match key_code {
            0x1D => {
                CTRL_PRESSED = true;
                None
            }
            0x2A | 0x36 => {
                SHIFT_PRESSED = true;
                None
            }
            0x38 => {
                ALT_PRESSED = true;
                None
            }
            _ => handle_scancode(key_code, is_extended)
        }
    }
}

fn handle_scancode(key_code: u8, is_extended: bool) -> Option<KeyEvents> {
    if is_extended {
        return match key_code {
            0x48 => Some(KeyEvents::ArrowUp),
            0x50 => Some(KeyEvents::ArrowDown),
            0x4B => {
                unsafe {
                    if CTRL_PRESSED {
                        Some(KeyEvents::SwitchScreenLeft)
                    } else {
                        Some(KeyEvents::ArrowLeft)
                    }
                }
            }
            0x4D => {
                unsafe {
                    if CTRL_PRESSED {
                        Some(KeyEvents::SwitchScreenRight)
                    } else {
                        Some(KeyEvents::ArrowRight)
                    }
                }
            }
            0x47 => Some(KeyEvents::Home),
            0x4F => Some(KeyEvents::End),
            0x53 => Some(KeyEvents::Delete),
            _ => None,
        };
    }
    
    if key_code < 128 {
        let ascii = SCANCODE_TO_ASCII[key_code as usize];
        if ascii != 0 {
            if ascii == 8 {
                return Some(KeyEvents::BackSpace);
            } else if ascii == b'\n' {
                return Some(KeyEvents::Enter);
            } else {
                let mut c = ascii as char;
                unsafe {
                    if SHIFT_PRESSED {
                        c = match c {
                            'a'..='z' => ((c as u8) - 32) as char,
                            '1' => '!',
                            '2' => '@',
                            '3' => '#',
                            '4' => '$',
                            '5' => '%',
                            '6' => '^',
                            '7' => '&',
                            '8' => '*',
                            '9' => '(',
                            '0' => ')',
                            '-' => '_',
                            '=' => '+',
                            '[' => '{',
                            ']' => '}',
                            '\\' => '|',
                            ';' => ':',
                            '\'' => '"',
                            ',' => '<',
                            '.' => '>',
                            '/' => '?',
                            '`' => '~',
                            _ => c,
                        };
                    }
                }
                return Some(KeyEvents::Character(c));
            }
        }
    }
    
    None
}

//=====================================================================================================================================
//                                         CURSOR  MANAGEMENT - REFACTORED TO USE SCREEN MANAGER
//=====================================================================================================================================

pub fn move_cursor_up() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    let row = active_screen.row_position;
    if row > 0 {
        active_screen.set_row_position(row - 1);
    }
    manager.update_cursor();
}

pub fn move_cursor_down() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    let row = active_screen.row_position;
    if row < BUFFER_HEIGHT - 1 {
        active_screen.set_row_position(row + 1);
    }
    manager.update_cursor();
}

pub fn move_cursor_left() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    let col = active_screen.column_position;
    if col > 0 {
        active_screen.set_column_position(col - 1);
    }
    manager.update_cursor();
}

pub fn move_cursor_right() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    let col = active_screen.column_position;
    if col < BUFFER_WIDTH - 1 {
        active_screen.set_column_position(col + 1);
    }
    manager.update_cursor();
}

pub fn write_at_cursor(c: char) {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    active_screen.write_byte(c as u8);
    manager.flush_to_physical();
    manager.update_cursor();
}

pub fn move_cursor_home() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    active_screen.set_cursor_position(0, 0);
    manager.update_cursor();
}

pub fn move_cursor_end() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    active_screen.set_cursor_position(BUFFER_HEIGHT - 1, BUFFER_WIDTH - 1);
    manager.update_cursor();
}

pub fn handle_backspace() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    let col = active_screen.column_position;
    if col > 0 {
        active_screen.set_column_position(col - 1);
        
        // Store cursor positions to avoid borrowing conflicts
        let row_pos = active_screen.row_position;
        let col_pos = active_screen.column_position;
        
        active_screen.write_byte_at(row_pos, col_pos, b' ');
    }
    manager.flush_to_physical();
    manager.update_cursor();
}

pub fn handle_delete() {
    let mut manager = screen_manager().lock();
    let active_screen = manager.get_active_screen_mut();
    
    // Store cursor positions to avoid borrowing conflicts
    let row_pos = active_screen.row_position;
    let col_pos = active_screen.column_position;
    
    active_screen.write_byte_at(row_pos, col_pos, b' ');
    manager.flush_to_physical();
    manager.update_cursor();
}