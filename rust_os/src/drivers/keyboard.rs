use crate::arch::x86::port::inb;
use crate::command::{init_command_handler, command_handler};
use crate::screen::global::{init_screen_manager, screen_manager};
use crate::screen::screen::{BUFFER_HEIGHT, BUFFER_WIDTH, Writer};
use crate::printk;

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

//=====================================================================================================================================
//                                         LISTEN TO KEYBOARD EVENTS
//=====================================================================================================================================

pub fn listen_to_keyboard_events() {
    if let Some(key_event) = poll_keyboard() {
        match key_event {
            KeyEvents::Character(c) => {
                let mut manager = screen_manager().lock();
                let mut cmd_handler = command_handler().lock();
                cmd_handler.add_char(c as u8, &mut manager);
            }
                
            KeyEvents::ArrowUp => {
                move_cursor_up();
            }
            KeyEvents::ArrowDown => {
                move_cursor_down();
            }
            KeyEvents::ArrowLeft => {
                let mut manager = screen_manager().lock();
                let mut cmd_handler = command_handler().lock();
                cmd_handler.move_cursor_left(&mut manager);
            }
            KeyEvents::ArrowRight => {
                let mut manager = screen_manager().lock();
                let mut cmd_handler = command_handler().lock();
                cmd_handler.move_cursor_right(&mut manager);
            }
            KeyEvents::Home => {
                let mut manager = screen_manager().lock();
                let mut cmd_handler = command_handler().lock();
                cmd_handler.move_cursor_home(&mut manager);
            }
            KeyEvents::End => {
                let mut manager = screen_manager().lock();
                let mut cmd_handler = command_handler().lock();
                cmd_handler.move_cursor_end(&mut manager);
            }
            KeyEvents::BackSpace => {
                let mut manager = screen_manager().lock();
                let mut cmd_handler = command_handler().lock();
                cmd_handler.backspace(&mut manager);
            }
            KeyEvents::Delete => {
                let mut manager = screen_manager().lock();
                let mut cmd_handler = command_handler().lock();
                cmd_handler.delete_char(&mut manager);
            }
            
            KeyEvents::Enter => {
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
                
            KeyEvents::SwitchScreenLeft => {
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
            KeyEvents::SwitchScreenRight => {
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