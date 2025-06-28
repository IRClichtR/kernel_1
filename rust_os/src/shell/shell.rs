use core::fmt::Write;
use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;
use crate::printk;

// Command buffer size - reasonable for kernel space
const COMMAND_BUFFER_SIZE: usize = 256;
const PROMPT: &str = "$> ";

#[derive(Debug)]
pub struct Shell {
    command_buffer: [u8; COMMAND_BUFFER_SIZE],
    buffer_length: usize,
    cursor_position: usize,
    prompt_displayed: bool,
}

impl Shell {
    pub const fn new() -> Self {
        Shell {
            command_buffer: [0; COMMAND_BUFFER_SIZE],
            buffer_length: 0,
            cursor_position: 0,
            prompt_displayed: false,
        }
    }

    pub fn init(&mut self) {
        self.clear_buffer();
        self.display_prompt();
    }

    pub fn display_prompt(&mut self) {
        if !self.prompt_displayed {
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
        if self.buffer_length < COMMAND_BUFFER_SIZE - 1 {
            // Insert character at cursor position
            if self.cursor_position < self.buffer_length {
                // Shift characters to the right
                for i in (self.cursor_position..self.buffer_length).rev() {
                    self.command_buffer[i + 1] = self.command_buffer[i];
                }
            }
            
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
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 && self.buffer_length > 0 {
            // Remove character at cursor position - 1
            self.cursor_position -= 1;
            
            // Shift characters to the left
            for i in self.cursor_position..self.buffer_length - 1 {
                self.command_buffer[i] = self.command_buffer[i + 1];
            }
            
            self.buffer_length -= 1;
            self.command_buffer[self.buffer_length] = 0;

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
        if self.cursor_position < self.buffer_length {
            // Shift characters to the left
            for i in self.cursor_position..self.buffer_length - 1 {
                self.command_buffer[i] = self.command_buffer[i + 1];
            }
            
            self.buffer_length -= 1;
            self.command_buffer[self.buffer_length] = 0;

            // Update display - clear character at current position
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
    }

    pub fn handle_enter(&mut self) {
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
        self.prompt_displayed = false;
        self.display_prompt();
    }

    pub fn handle_arrow_left(&mut self) {
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

        // Parse command and arguments
        let mut parts = command_str.trim().split_whitespace();
        let command = parts.next().unwrap_or("");
        
        match command {
            "help" => self.cmd_help(),
            "clear" => self.cmd_clear(),
            "echo" => self.cmd_echo(parts.collect::<Vec<&str>>().join(" ")),
            "screen" => {
                if let Some(arg) = parts.next() {
                    self.cmd_screen(arg);
                } else {
                    self.print_line("Usage: screen <id>");
                }
            },
            "status" => self.cmd_status(),
            "" => {}, // Empty command, do nothing
            _ => {
                self.print_error("Unknown command: ");
                self.print_line(command);
            }
        }
    }

    fn cmd_help(&mut self) {
        self.print_line("Available commands:");
        self.print_line("  help     - Show this help message");
        self.print_line("  clear    - Clear the screen");
        self.print_line("  echo     - Display text");
        self.print_line("  screen   - Switch to screen <id>");
        self.print_line("  status   - Show system status");
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

    fn cmd_echo(&mut self, text: &str) {
        if !text.is_empty() {
            self.print_line(text);
        }
    }

    fn cmd_screen(&mut self, arg: &str) {
        if let Ok(screen_id) = arg.parse::<usize>() {
            let switch_successful = {
                let mut manager = screen_manager().lock();
                manager.switch_screen(screen_id)
            };
            
            if switch_successful {
                // Use a simple string concatenation approach for kernel space
                let mut msg = "Switched to screen ";
                self.print_simple_message(msg, screen_id);
            } else {
                let mut msg = "Failed to switch to screen ";
                self.print_simple_error(msg, screen_id);
            }
        } else {
            self.print_error("Invalid screen ID. Use a number.");
        }
    }

    fn cmd_status(&mut self) {
        let manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        let total_screens = manager.screens.iter().filter(|s| s.is_some()).count();
        
        // Use simple string formatting for kernel space
        self.print_simple_message("Active screen: ", active_screen_id);
        self.print_simple_message("Total screens: ", total_screens);
        self.print_line("System: Rust OS Shell v1.0");
    }

    fn print_line(&mut self, text: &str) {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            write!(writer, "{}\n", text).unwrap();
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }

    fn print_error(&mut self, text: &str) {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            write!(writer, "Error: {}\n", text).unwrap();
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }

    fn print_simple_message(&mut self, prefix: &str, number: usize) {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            write!(writer, "{}{}\n", prefix, number).unwrap();
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }

    fn print_simple_error(&mut self, prefix: &str, number: usize) {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.active_screen_id;
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            write!(writer, "Error: {}{}\n", prefix, number).unwrap();
        }
        manager.flush_to_physical();
        manager.update_cursor();
    }
}

// Global shell instance
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