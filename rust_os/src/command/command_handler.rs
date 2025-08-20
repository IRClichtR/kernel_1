use crate::screen::global::screen_manager;
use crate::screen::screen::{Writer, BUFFER_WIDTH};
use crate::arch::x86::port::outb;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Reboot,
    Halt,
    Clear,
    Help,
    Unknown,
}

pub struct CommandHandler {
    buffer: [u8; 256],
    buffer_len: usize,
    prompt_start_col: usize, 
    prompt_start_row: usize,
}

impl CommandHandler {
    pub const fn new() -> Self {
        Self {
            buffer: [0; 256],
            buffer_len: 0,
            prompt_start_col: 0,
            prompt_start_row: 0,
        }
    }

    pub fn set_prompt_position(&mut self, row: usize, col: usize) {
        self.prompt_start_row = row;
        self.prompt_start_col = col;
    }

    pub fn add_char(&mut self, ch: u8, manager: &mut crate::screen::manager::ScreenManager) {
        if self.buffer_len < self.buffer.len() - 1 && ch != b'\n' {
            if let Some(screen) = manager.get_screen_mut(2) {
                let cursor_pos = screen.column_position.saturating_sub(self.prompt_start_col);
                
                if cursor_pos < self.buffer_len {
                    for i in (cursor_pos..self.buffer_len).rev() {
                        self.buffer[i + 1] = self.buffer[i];
                    }
                }
                
                self.buffer[cursor_pos] = ch;
                self.buffer_len += 1;
                
                if cursor_pos == self.buffer_len - 1 {
                    let mut writer = Writer::new(screen);
                    writer.write_byte(ch);
                } else {
                    let start_col = self.prompt_start_col + cursor_pos;
                    let current_row = screen.row_position;
                    
                    for i in cursor_pos..self.buffer_len {
                        let col = start_col + (i - cursor_pos);
                        if col < BUFFER_WIDTH {
                            screen.write_byte_at(current_row, col, self.buffer[i]);
                        }
                    }
                    
                    screen.column_position = self.prompt_start_col + cursor_pos + 1;
                }
                
                if manager.get_active_screen_id() == 2 {
                    manager.flush_to_physical();
                    manager.update_cursor();
                }
            }
        }
    }

    pub fn delete_char(&mut self, manager: &mut crate::screen::manager::ScreenManager) {
        if let Some(screen) = manager.get_screen_mut(2) {
            let cursor_pos = screen.column_position.saturating_sub(self.prompt_start_col);
            
            if cursor_pos < self.buffer_len {
                for i in cursor_pos..self.buffer_len - 1 {
                    self.buffer[i] = self.buffer[i + 1];
                }
                
                self.buffer_len -= 1;
                self.buffer[self.buffer_len] = 0;
                
                let row_pos = screen.row_position;
                let col_pos = screen.column_position;
                
                screen.write_byte_at(row_pos, col_pos, b' ');
                
                for i in 0..self.buffer_len - cursor_pos {
                    let col = self.prompt_start_col + cursor_pos + i;
                    if col < BUFFER_WIDTH {
                        screen.write_byte_at(row_pos, col, self.buffer[cursor_pos + i]);
                    }
                }
                
                let trailing_col = self.prompt_start_col + self.buffer_len;
                if trailing_col < BUFFER_WIDTH {
                    screen.write_byte_at(row_pos, trailing_col, b' ');
                }
            }
            
            if manager.get_active_screen_id() == 2 {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
    }

    pub fn backspace(&mut self, manager: &mut crate::screen::manager::ScreenManager) {
        if let Some(screen) = manager.get_screen_mut(2) {
            let cursor_pos = screen.column_position.saturating_sub(self.prompt_start_col);
            
            if cursor_pos > 0 {
                screen.column_position -= 1;
                
                for i in cursor_pos - 1..self.buffer_len - 1 {
                    self.buffer[i] = self.buffer[i + 1];
                }
                
                self.buffer_len -= 1;
                self.buffer[self.buffer_len] = 0;
                
                let row_pos = screen.row_position;
                let col_pos = screen.column_position;
                
                screen.write_byte_at(row_pos, col_pos, b' ');
                
                for i in 0..self.buffer_len - (cursor_pos - 1) {
                    let col = self.prompt_start_col + (cursor_pos - 1) + i;
                    if col < BUFFER_WIDTH {
                        screen.write_byte_at(row_pos, col, self.buffer[(cursor_pos - 1) + i]);
                    }
                }
                
                let trailing_col = self.prompt_start_col + self.buffer_len;
                if trailing_col < BUFFER_WIDTH {
                    screen.write_byte_at(row_pos, trailing_col, b' ');
                }
            }
            
            if manager.get_active_screen_id() == 2 {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
    }

    pub fn move_cursor_left(&mut self, manager: &mut crate::screen::manager::ScreenManager) {
        if let Some(screen) = manager.get_screen_mut(2) {
            let cursor_pos = screen.column_position.saturating_sub(self.prompt_start_col);
            if cursor_pos > 0 {
                screen.column_position -= 1;
            }
            
            if manager.get_active_screen_id() == 2 {
                manager.update_cursor();
            }
        }
    }

    pub fn move_cursor_right(&mut self, manager: &mut crate::screen::manager::ScreenManager) {
        if let Some(screen) = manager.get_screen_mut(2) {
            let cursor_pos = screen.column_position.saturating_sub(self.prompt_start_col);
            if cursor_pos < self.buffer_len {
                screen.column_position += 1;
            }
            
            if manager.get_active_screen_id() == 2 {
                manager.update_cursor();
            }
        }
    }

    pub fn move_cursor_home(&mut self, manager: &mut crate::screen::manager::ScreenManager) {
        if let Some(screen) = manager.get_screen_mut(2) {
            screen.column_position = self.prompt_start_col;
            
            if manager.get_active_screen_id() == 2 {
                manager.update_cursor();
            }
        }
    }

    pub fn move_cursor_end(&mut self, manager: &mut crate::screen::manager::ScreenManager) {
        if let Some(screen) = manager.get_screen_mut(2) {
            screen.column_position = self.prompt_start_col.saturating_add(self.buffer_len);
            
            if manager.get_active_screen_id() == 2 {
                manager.update_cursor();
            }
        }
    }

    pub fn execute_command(&mut self) -> bool {
        if self.buffer_len == 0 {
            return false;
        }

        let command_str = core::str::from_utf8(&self.buffer[..self.buffer_len])
            .unwrap_or("");

        let command = self.parse_command(command_str);
        self.handle_command(command);
        
        self.clear_buffer();
        true
    }

    fn parse_command(&self, input: &str) -> Command {
        match input.trim() {
            "reboot" => Command::Reboot,
            "clear" => Command::Clear,
            "help" => Command::Help,
            "halt" => Command::Halt,
            _ => Command::Unknown,
        }
    }

    fn handle_command(&self, command: Command) {
        match command {
            Command::Reboot => {
                self.execute_reboot();
            }
            Command::Clear => {
                self.execute_clear();
            }
            Command::Help => {
                self.execute_help();
            }
            Command::Halt => {
                self.execute_halt();
            }
            Command::Unknown => {
                self.execute_unknown();
            }
        }
    }

    fn execute_clear(&self) {
        let mut manager = screen_manager().lock();
        if manager.clear_screen(2) {
            if let Some(screen) = manager.get_screen_mut(2) {
                screen.set_cursor_position(0, 0);
            }
            if manager.get_active_screen_id() == 2 {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
    }

    fn execute_help(&self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = manager.get_screen_mut(2) {
            let mut writer = Writer::new(screen);
            for byte in b"Available commands:\n" {
                writer.write_byte(*byte);
            }
            for byte in b"  help   - Show this help message\n" {
                writer.write_byte(*byte);
            }
            for byte in b"  clear  - Clear the screen\n" {
                writer.write_byte(*byte);
            }
            for byte in b"  reboot - Restart the system\n" {
                writer.write_byte(*byte);
            }
            for byte in b"  halt   - Halt the system (safe to power off)\n" {
                writer.write_byte(*byte);
            }
            writer.write_byte(b'\n');
            
            if manager.get_active_screen_id() == 2 {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
    }

    fn execute_unknown(&self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = manager.get_screen_mut(2) {
            let mut writer = Writer::new(screen);
            for byte in b"Unknown command. Type 'help' for available commands.\n" {
                writer.write_byte(*byte);
            }
            
            if manager.get_active_screen_id() == 2 {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
    }

    fn execute_reboot(&self) {
        {
            let mut manager = screen_manager().lock();
            if let Some(screen) = manager.get_screen_mut(2) {
                let mut writer = Writer::new(screen);
                for byte in b"Rebooting system...\n" {
                    writer.write_byte(*byte);
                }
                
                if manager.get_active_screen_id() == 2 {
                    manager.flush_to_physical();
                    manager.update_cursor();
                }
            }
        }

        unsafe {
            while (crate::arch::x86::port::inb(0x64) & 0x02) != 0 {}
            outb(0x64, 0xFE);
        }

        unsafe {
            #[repr(C, packed)]
            struct InvalidIDT {
                limit: u16,
                base: u64,
            }
            
            let invalid_idt = InvalidIDT {
                limit: 0,
                base: 0,
            };
            
            core::arch::asm!(
                "lidt [{}]",
                in(reg) &invalid_idt,
                options(nostack, preserves_flags)
            );
            
            core::arch::asm!("int 3", options(nostack, preserves_flags));
        }
    }

    fn execute_halt(&self) {
        {
            let mut manager = screen_manager().lock();
            if let Some(screen) = manager.get_screen_mut(2) {
                let mut writer = Writer::new(screen);
                for byte in b"System halted. Safe to power off.\n" {
                    writer.write_byte(*byte);
                }
                
                if manager.get_active_screen_id() == 2 {
                    manager.flush_to_physical();
                    manager.update_cursor();
                }
            }
        }

        unsafe {
            core::arch::asm!(
                "cli",
                "hlt",
                options(nostack, preserves_flags)
            );
        }

        loop {
            unsafe {
                core::arch::asm!(
                    "hlt",
                    options(nostack, preserves_flags)
                );
            }
        }
    }

    fn clear_buffer(&mut self) {
        self.buffer_len = 0;
        for i in 0..self.buffer.len() {
            self.buffer[i] = 0;
        }
    }
}