use core::fmt::Write;
use core::arch::asm;
use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;
use crate::printk;
use crate::printk::printk::LogLevel;

// Command buffer size - reasonable for kernel space
const COMMAND_BUFFER_SIZE: usize = 256;
const PROMPT: &str = "$> ";

// Input state tracking for better error handling
#[derive(Debug, Clone, Copy, PartialEq)]
enum InputState {
    Ready,
    Processing,
    Error,
}

#[derive(Debug)]
pub struct Shell {
    command_buffer: [u8; COMMAND_BUFFER_SIZE],
    buffer_length: usize,
    cursor_position: usize,
    prompt_displayed: bool,
    input_state: InputState,
    last_error: Option<&'static str>,
}

impl Shell {
    pub const fn new() -> Self {
        Shell {
            command_buffer: [0; COMMAND_BUFFER_SIZE],
            buffer_length: 0,
            cursor_position: 0,
            prompt_displayed: false,
            input_state: InputState::Ready,
            last_error: None,
        }
    }

    // Check if we're on the user terminal screen (screen 1)
    fn is_on_user_terminal(&self) -> bool {
        let manager = screen_manager().lock();
        manager.active_screen_id == 1
    }

    // Enhanced input validation
    fn is_valid_input_char(&self, c: char) -> bool {
        // Allow printable ASCII characters and common control characters
        matches!(c, 
            ' '..='~' | // Printable ASCII
            '\t' |      // Tab
            '\n' |      // Newline (though handled separately)
            '\r'        // Carriage return (though handled separately)
        )
    }

    // Check if buffer is full
    fn is_buffer_full(&self) -> bool {
        self.buffer_length >= COMMAND_BUFFER_SIZE - 1
    }

    // Check if we can add more characters
    fn can_add_character(&self) -> bool {
        !self.is_buffer_full() && self.input_state == InputState::Ready
    }

    pub fn init(&mut self) {
        self.clear_buffer();
        self.reset_input_state();
        self.display_prompt();
    }

    fn reset_input_state(&mut self) {
        self.input_state = InputState::Ready;
        self.last_error = None;
    }

    fn set_error(&mut self, error: &'static str) {
        self.input_state = InputState::Error;
        self.last_error = Some(error);
    }

    pub fn display_prompt(&mut self) {
        if !self.prompt_displayed && self.is_on_user_terminal() {
            let mut manager = screen_manager().lock();
            let active_screen_id = manager.active_screen_id;
            if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                let mut writer = Writer::new(active_screen);
                write!(writer, "{}", PROMPT).unwrap();
            }
            manager.flush_to_physical();
            manager.update_cursor();
            self.prompt_displayed = true;
        }
    }

    pub fn handle_character(&mut self, c: char) {
        if !self.is_on_user_terminal() {
            return; // Only handle characters on user terminal
        }

        // Enhanced input validation
        if !self.is_valid_input_char(c) {
            self.set_error("Invalid character");
            return;
        }

        if !self.can_add_character() {
            if self.is_buffer_full() {
                self.set_error("Buffer full");
            }
            return;
        }

        // Set processing state
        self.input_state = InputState::Processing;

        // Validate cursor position
        if self.cursor_position > self.buffer_length {
            self.cursor_position = self.buffer_length;
        }

        // Insert character at cursor position with bounds checking
        if self.cursor_position < self.buffer_length {
            // Shift characters to the right with bounds checking
            for i in (self.cursor_position..self.buffer_length).rev() {
                if i + 1 < COMMAND_BUFFER_SIZE {
                    self.command_buffer[i + 1] = self.command_buffer[i];
                }
            }
        }
        
        // Add character with bounds checking
        if self.cursor_position < COMMAND_BUFFER_SIZE {
            self.command_buffer[self.cursor_position] = c as u8;
            self.buffer_length += 1;
            self.cursor_position += 1;

            // Display character on screen
            let mut manager = screen_manager().lock();
            let active_screen_id = manager.active_screen_id;
            if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                let mut writer = Writer::new(active_screen);
                writer.write_byte(c as u8);
            }
            manager.flush_to_physical();
            manager.update_cursor();
        }

        // Reset to ready state
        self.input_state = InputState::Ready;
    }

    pub fn handle_backspace(&mut self) {
        if !self.is_on_user_terminal() {
            return; // Only handle backspace on user terminal
        }
        
        if self.input_state != InputState::Ready {
            return; // Don't process if in error state
        }

        if self.cursor_position > 0 && self.buffer_length > 0 {
            // Remove character at cursor position - 1
            self.cursor_position -= 1;
            
            // Shift characters to the left with bounds checking
            for i in self.cursor_position..self.buffer_length - 1 {
                if i < COMMAND_BUFFER_SIZE - 1 {
                    self.command_buffer[i] = self.command_buffer[i + 1];
                }
            }
            
            self.buffer_length -= 1;
            if self.buffer_length < COMMAND_BUFFER_SIZE {
                self.command_buffer[self.buffer_length] = 0;
            }

            // Update display - move cursor back and clear character
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
    }

    pub fn handle_delete(&mut self) {
        if !self.is_on_user_terminal() {
            return; // Only handle delete on user terminal
        }
        
        if self.input_state != InputState::Ready {
            return; // Don't process if in error state
        }
        
        if self.cursor_position < self.buffer_length {
            // Shift characters to the left starting from cursor position with bounds checking
            for i in self.cursor_position..self.buffer_length - 1 {
                if i < COMMAND_BUFFER_SIZE - 1 {
                    self.command_buffer[i] = self.command_buffer[i + 1];
                }
            }
            
            self.buffer_length -= 1;
            if self.buffer_length < COMMAND_BUFFER_SIZE {
                self.command_buffer[self.buffer_length] = 0;
            }

            // Update display - clear character at current position and shift remaining characters
            let mut manager = screen_manager().lock();
            let active_screen_id = manager.active_screen_id;
            if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                let row = active_screen.row_position;
                let col = active_screen.column_position;
                
                // Clear current character
                active_screen.write_byte_at(row, col, b' ');
                
                // Shift remaining characters on screen to the left
                for i in 0..(self.buffer_length - self.cursor_position) {
                    if self.cursor_position + i < COMMAND_BUFFER_SIZE {
                        let next_char = self.command_buffer[self.cursor_position + i];
                        active_screen.write_byte_at(row, col + i, next_char);
                    }
                }
                
                // Clear the last character that was shifted
                active_screen.write_byte_at(row, col + (self.buffer_length - self.cursor_position), b' ');
            }
            manager.flush_to_physical();
            manager.update_cursor();
        }
    }

    pub fn handle_enter(&mut self) {
        if !self.is_on_user_terminal() {
            return; // Only handle enter on user terminal
        }
        
        // Don't process if in error state
        if self.input_state == InputState::Error {
            self.reset_input_state();
            return;
        }
        
        // Add newline after command
        {
            let mut manager = screen_manager().lock();
            let active_screen_id = manager.active_screen_id;
            if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                let mut writer = Writer::new(active_screen);
                writer.write_byte(b'\n');
            }
            manager.flush_to_physical();
            manager.update_cursor();
        }

        // Execute command if buffer is not empty
        if self.buffer_length > 0 {
            self.execute_command();
        }

        // Reset shell state
        self.clear_buffer();
        self.reset_input_state();
        self.prompt_displayed = false;
        self.display_prompt();
    }

    pub fn handle_arrow_left(&mut self) {
        if !self.is_on_user_terminal() {
            return;
        }
        
        if self.input_state != InputState::Ready {
            return; // Don't process if in error state
        }
        
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            
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
    }

    pub fn handle_arrow_right(&mut self) {
        if !self.is_on_user_terminal() {
            return;
        }

        if self.input_state != InputState::Ready {
            return; // Don't process if in error state
        }

        if self.cursor_position < self.buffer_length {
            self.cursor_position += 1;
            
            let mut manager = screen_manager().lock();
            let active_screen_id = manager.active_screen_id;
            if let Some(active_screen) = &mut manager.screens[active_screen_id] {
                let col = active_screen.column_position;
                if col < crate::screen::screen::BUFFER_WIDTH - 1 {
                    active_screen.set_column_position(col + 1);
                }
            }
            manager.update_cursor();
        }
    }

    pub fn handle_home(&mut self) {
        if !self.is_on_user_terminal() {
            return;
        }

        if self.input_state != InputState::Ready {
            return; // Don't process if in error state
        }

        self.cursor_position = 0;
        
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let row = active_screen.row_position;
            active_screen.set_cursor_position(row, PROMPT.len());
        }
        manager.update_cursor();
    }

    pub fn handle_end(&mut self) {
        if !self.is_on_user_terminal() {
            return;
        }
        
        if self.input_state != InputState::Ready {
            return; // Don't process if in error state
        }
        
        self.cursor_position = self.buffer_length;
        
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let row = active_screen.row_position;
            let col = PROMPT.len() + self.buffer_length;
            if col < crate::screen::screen::BUFFER_WIDTH {
                active_screen.set_cursor_position(row, col);
            }
        }
        manager.update_cursor();
    }

    fn clear_buffer(&mut self) {
        self.command_buffer.fill(0);
        self.buffer_length = 0;
        self.cursor_position = 0;
    }

    fn execute_command(&mut self) {
        // Convert buffer to string for parsing
        let command_str = core::str::from_utf8(&self.command_buffer[..self.buffer_length])
            .unwrap_or("");
        
        // Trim whitespace
        let command_str = command_str.trim();
        
        // Parse command and arguments
        let mut parts = command_str.split_whitespace();
        let command = parts.next().unwrap_or("");
        
        match command {
            "reboot" => self.cmd_reboot(),
            "halt" => self.cmd_halt(),
            "clear" => self.cmd_clear(),
            "help" => self.cmd_help(),
            "" => {}, // Empty command, do nothing
            _ => self.cmd_unknown(command),
        }
    }

    fn cmd_reboot(&mut self) {
        self.print_message("Rebooting system...\n");
        
        // Flush any pending output
        {
            let mut manager = screen_manager().lock();
            manager.flush_to_physical();
        }
        
        // Wait a moment for the message to be displayed
        self.delay_ms(1000);
        
        // Attempt reboot using multiple methods
        self.reboot_system();
    }

    fn cmd_halt(&mut self) {
        self.print_message("System halted. You can safely power off.\n");
        
        // Flush any pending output
        {
            let mut manager = screen_manager().lock();
            manager.flush_to_physical();
        }
        
        // Halt the system
        self.halt_system();
    }

    fn cmd_clear(&mut self) {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            active_screen.clear();
            active_screen.set_cursor_position(0, 0);
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }

    fn cmd_help(&mut self) {
        self.print_message("Available commands:\n");
        self.print_message("  help    - Show this help message\n");
        self.print_message("  clear   - Clear the screen\n");
        self.print_message("  reboot  - Restart the system\n");
        self.print_message("  halt    - Halt the system\n");
    }

    fn cmd_unknown(&mut self, command: &str) {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            write!(writer, "Unknown command: {}\n", command).unwrap();
            write!(writer, "Type 'help' for available commands.\n").unwrap();
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }

    fn print_message(&mut self, message: &str) {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            write!(writer, "{}", message).unwrap();
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }

    fn reboot_system(&self) {
        // Method 1: Try ACPI reset (most modern systems)
        unsafe {
            // ACPI reset register (if available)
            asm!("
                mov dx, 0xcf9
                mov al, 0x02
                out dx, al
                mov al, 0x06
                out dx, al
            ");
        }

        // Small delay
        self.delay_ms(100);

        // Method 2: Keyboard controller reset (older systems)
        unsafe {
            // Reset via keyboard controller
            asm!("
                mov dx, 0x64
                wait_kb1:
                    in al, dx
                    test al, 0x02
                    jnz wait_kb1
                mov al, 0xfe
                out dx, al
            ");
        }

        // Small delay
        self.delay_ms(100);

        // Method 3: Triple fault (force CPU reset)
        unsafe {
            // Load invalid IDT to cause triple fault
            asm!("
                lidt [{}]
                int 3
            ", in(reg) &0u64 as *const u64);
        }
    }

    fn halt_system(&self) -> ! {
        // Disable interrupts
        unsafe {
            asm!("cli");
        }

        // Halt the CPU
        loop {
            unsafe {
                asm!("hlt");
            }
        }
    }

    fn delay_ms(&self, ms: u32) {
        // Simple delay loop - adjust multiplier based on your CPU speed
        let cycles = ms * 1000000; // Rough estimate
        for _ in 0..cycles {
            unsafe {
                asm!("nop");
            }
        }
    }
}

use crate::kspin_lock::kspin_lock::KSpinLock;

static SHELL: KSpinLock<Shell> = KSpinLock::new(Shell::new());

pub fn global_shell() -> &'static KSpinLock<Shell> {
    &SHELL
}

pub fn init_shell() {
    let mut shell = SHELL.lock();
    shell.init();
    printk!(LogLevel::Info, "Shell initialized.\n");
}