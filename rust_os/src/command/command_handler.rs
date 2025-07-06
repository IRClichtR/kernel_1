use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;
use crate::arch::x86::port::outb;
use core::fmt::Write;

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
    cursor_pos: usize,
    prompt_start_col: usize, 
    prompt_start_row: usize,
}

impl CommandHandler {
    pub const fn new() -> Self {
        Self {
            buffer: [0; 256],
            buffer_len: 0,
            cursor_pos: 0,
            prompt_start_col: 0,
            prompt_start_row: 0,
        }
    }

    pub fn set_prompt_position(&mut self, row: usize, col: usize) {
        self.prompt_start_row = row;
        self.prompt_start_col = col;
    }

    pub fn add_char(&mut self, ch: u8) {
        if self.buffer_len < self.buffer.len() - 1 && ch != b'\n' {
            if self.cursor_pos < self.buffer_len {
                for i in (self.cursor_pos..self.buffer_len).rev() {
                    self.buffer[i + 1] = self.buffer[i];
                }
            }
            
            self.buffer[self.cursor_pos] = ch;
            self.buffer_len += 1;
            self.cursor_pos += 1;
            self.refresh_command_display();
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos < self.buffer_len {
            for i in self.cursor_pos..self.buffer_len - 1 {
                self.buffer[i] = self.buffer[i + 1];
            }
            
            self.buffer_len -= 1;
            self.buffer[self.buffer_len] = 0;
            
            self.refresh_command_display();
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.delete_char();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.update_screen_cursor();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.buffer_len {
            self.cursor_pos += 1;
            self.update_screen_cursor();
        }
    }


    pub fn move_cursor_home(&mut self) {
        self.cursor_pos = 0;
        self.update_screen_cursor();
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.buffer_len;
        self.update_screen_cursor();
    }

    fn refresh_command_display(&mut self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = &mut manager.screens[1] {
            let _saved_column_position = screen.column_position;
            let _saved_row_position = screen.row_position;
            
            screen.column_position = self.prompt_start_col;
            screen.row_position = self.prompt_start_row;
            
            {
                let mut writer = Writer::new(screen);
                for _ in 0..(80 - self.prompt_start_col) {
                    writer.write_byte(b' ');
                }
            } // Writer is dropped here, releasing the borrow
            
            // Reset to prompt position and redraw command
            screen.column_position = self.prompt_start_col;
            screen.row_position = self.prompt_start_row;
            
            // Write the current command buffer
            {
                let mut writer = Writer::new(screen);
                for i in 0..self.buffer_len {
                    writer.write_byte(self.buffer[i]);
                }
            } // Writer is dropped here, releasing the borrow
            
            // Position cursor at the correct location
            screen.column_position = self.prompt_start_col + self.cursor_pos;
            screen.row_position = self.prompt_start_row;
        }
        
        manager.flush_to_physical();
        manager.update_cursor();
    }

    fn update_screen_cursor(&mut self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = &mut manager.screens[1] {
            screen.column_position = self.prompt_start_col + self.cursor_pos;
            screen.row_position = self.prompt_start_row;
        }
        manager.update_cursor();
    }

    pub fn execute_command(&mut self) -> bool {
        if self.buffer_len == 0 {
            return false;
        }

        // Convert buffer to string slice for comparison
        let command_str = core::str::from_utf8(&self.buffer[..self.buffer_len])
            .unwrap_or("");

        let command = self.parse_command(command_str);
        self.handle_command(command);
        
        // Clear the buffer after execution
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

    /// Executes the reboot command
    fn execute_reboot(&self) {
        // Write message before rebooting
        {
            let mut manager = screen_manager().lock();
            if let Some(screen) = &mut manager.screens[1] {
                let mut writer = Writer::new(screen);
                write!(writer, "Rebooting system...\n").unwrap();
            }
            manager.flush_to_physical();
        }

        // Perform keyboard controller reset (8042 reset)
        unsafe {
            // Wait for keyboard controller to be ready
            while (crate::arch::x86::port::inb(0x64) & 0x02) != 0 {}
            
            // Send reset command
            outb(0x64, 0xFE);
        }

        // If that doesn't work, try triple fault
        unsafe {
            // Load invalid IDT to cause triple fault
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
            
            // Trigger interrupt with invalid IDT
            core::arch::asm!("int 3", options(nostack, preserves_flags));
        }
    }

    fn execute_halt(&self) {
        {
            let mut manager = screen_manager().lock();
            if let Some(screen) = &mut manager.screens[1] {
                let mut writer = Writer::new(screen);
                write!(writer, "System halted. Safe to power off.\n").unwrap();
            }
            manager.flush_to_physical();
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

    fn execute_clear(&self) {
        let mut manager = screen_manager().lock();
        if manager.clear_screen(1) {
            // Reset cursor to top
            manager.set_cursor_position(0, 0);
            manager.flush_to_physical();
        }
    }

    /// Executes the help command
    fn execute_help(&self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = &mut manager.screens[1] {
            let mut writer = Writer::new(screen);
            write!(writer, "Available commands:\n").unwrap();
            write!(writer, "  help   - Show this help message\n").unwrap();
            write!(writer, "  clear  - Clear the screen\n").unwrap();
            write!(writer, "  reboot - Restart the system\n").unwrap();
            write!(writer, "  halt   - Halt the system (safe to power off)\n").unwrap();
            write!(writer, "\n").unwrap();
        }
        manager.flush_to_physical();
    }

    /// Executes unknown command response
    fn execute_unknown(&self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = &mut manager.screens[1] {
            let mut writer = Writer::new(screen);
            write!(writer, "Unknown command. Type 'help' for available commands.\n").unwrap();
        }
        manager.flush_to_physical();
    }

    /// Clears the command buffer and resets cursor position
    fn clear_buffer(&mut self) {
        self.buffer_len = 0;
        self.cursor_pos = 0;
        for i in 0..self.buffer.len() {
            self.buffer[i] = 0;
        }
    }

    /// Returns the current buffer length
    pub fn get_buffer_len(&self) -> usize {
        self.buffer_len
    }

    /// Returns the current cursor position within the buffer
    pub fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    /// Gets the current command as a string slice
    pub fn get_command_str(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.buffer_len]).unwrap_or("")
    }
}