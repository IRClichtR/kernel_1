use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;
use crate::arch::x86::port::outb;
use core::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Reboot,
    Clear,
    Help,
    Unknown,
}

pub struct CommandHandler {
    buffer: [u8; 256],
    buffer_len: usize,
}

impl CommandHandler {
    pub const fn new() -> Self {
        Self {
            buffer: [0; 256],
            buffer_len: 0,
        }
    }

    pub fn add_char(&mut self, ch: u8) {
        if self.buffer_len < self.buffer.len() - 1 && ch != b'\n' {
            self.buffer[self.buffer_len] = ch;
            self.buffer_len += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.buffer_len > 0 {
            self.buffer_len -= 1;
            self.buffer[self.buffer_len] = 0;
        }
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
            Command::Unknown => {
                self.execute_unknown();
            }
        }
    }

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

    fn execute_clear(&self) {
        let mut manager = screen_manager().lock();
        if manager.clear_screen(1) {
            // Reset cursor to top
            manager.set_cursor_position(0, 0);
            manager.flush_to_physical();
        }
    }

    fn execute_help(&self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = &mut manager.screens[1] {
            let mut writer = Writer::new(screen);
            write!(writer, "Available commands:\n").unwrap();
            write!(writer, "  help   - Show this help message\n").unwrap();
            write!(writer, "  clear  - Clear the screen\n").unwrap();
            write!(writer, "  reboot - Restart the system\n").unwrap();
            write!(writer, "\n").unwrap();
        }
        manager.flush_to_physical();
    }

    fn execute_unknown(&self) {
        let mut manager = screen_manager().lock();
        if let Some(screen) = &mut manager.screens[1] {
            let mut writer = Writer::new(screen);
            write!(writer, "Unknown command. Type 'help' for available commands.\n").unwrap();
        }
        manager.flush_to_physical();
    }

    fn clear_buffer(&mut self) {
        self.buffer_len = 0;
        for i in 0..self.buffer.len() {
            self.buffer[i] = 0;
        }
    }

    pub fn get_buffer_len(&self) -> usize {
        self.buffer_len
    }
}