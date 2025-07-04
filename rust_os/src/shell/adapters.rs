// src/shell/adapters.rs

#![no_std]

use crate::drivers::keyboard::{KeyEvent, get_key_event};
use crate::screen::{SCREEN_MANAGER, ScreenManager};
use super::interfaces::*;

/// Input adapter for kernel_1's keyboard system
pub struct Kernel1InputAdapter {
    char_buffer: [char; 32],
    buffer_start: usize,
    buffer_end: usize,
    buffer_count: usize,
}

impl Kernel1InputAdapter {
    pub const fn new() -> Self {
        Self {
            char_buffer: ['\0'; 32],
            buffer_start: 0,
            buffer_end: 0,
            buffer_count: 0,
        }
    }
    
    /// Update the input adapter with new keyboard events
    pub fn update(&mut self) {
        if let Some(key_event) = get_key_event() {
            if let Some(ch) = self.key_event_to_char(key_event) {
                self.push_char(ch);
            }
        }
    }
    
    fn key_event_to_char(&self, key_event: KeyEvent) -> Option<char> {
        match key_event {
            KeyEvent::KeyPressed(scancode) => {
                // Convert scancode to character
                // This is a simplified conversion - you may need to expand this
                match scancode {
                    0x1C => Some('\n'),  // Enter
                    0x0E => Some('\x08'), // Backspace
                    0x39 => Some(' '),   // Space
                    0x1E => Some('a'),
                    0x30 => Some('b'),
                    0x2E => Some('c'),
                    0x20 => Some('d'),
                    0x12 => Some('e'),
                    0x21 => Some('f'),
                    0x22 => Some('g'),
                    0x23 => Some('h'),
                    0x17 => Some('i'),
                    0x24 => Some('j'),
                    0x25 => Some('k'),
                    0x26 => Some('l'),
                    0x32 => Some('m'),
                    0x31 => Some('n'),
                    0x18 => Some('o'),
                    0x19 => Some('p'),
                    0x10 => Some('q'),
                    0x13 => Some('r'),
                    0x1F => Some('s'),
                    0x14 => Some('t'),
                    0x16 => Some('u'),
                    0x2F => Some('v'),
                    0x11 => Some('w'),
                    0x2D => Some('x'),
                    0x15 => Some('y'),
                    0x2C => Some('z'),
                    0x02 => Some('1'),
                    0x03 => Some('2'),
                    0x04 => Some('3'),
                    0x05 => Some('4'),
                    0x06 => Some('5'),
                    0x07 => Some('6'),
                    0x08 => Some('7'),
                    0x09 => Some('8'),
                    0x0A => Some('9'),
                    0x0B => Some('0'),
                    _ => None,
                }
            }
            KeyEvent::KeyReleased(_) => None,
        }
    }
    
    fn push_char(&mut self, ch: char) {
        if self.buffer_count < self.char_buffer.len() {
            self.char_buffer[self.buffer_end] = ch;
            self.buffer_end = (self.buffer_end + 1) % self.char_buffer.len();
            self.buffer_count += 1;
        }
    }
    
    fn pop_char(&mut self) -> Option<char> {
        if self.buffer_count > 0 {
            let ch = self.char_buffer[self.buffer_start];
            self.buffer_start = (self.buffer_start + 1) % self.char_buffer.len();
            self.buffer_count -= 1;
            Some(ch)
        } else {
            None
        }
    }
}

impl InputProvider for Kernel1InputAdapter {
    fn get_next_char(&mut self) -> Option<char> {
        self.update();
        self.pop_char()
    }
    
    fn has_input(&self) -> bool {
        self.buffer_count > 0
    }
    
    fn reset(&mut self) {
        self.buffer_start = 0;
        self.buffer_end = 0;
        self.buffer_count = 0;
    }
}

/// Output adapter for kernel_1's screen manager
pub struct Kernel1OutputAdapter;

impl Kernel1OutputAdapter {
    pub const fn new() -> Self {
        Self
    }
}

impl OutputProvider for Kernel1OutputAdapter {
    fn write_char(&mut self, c: char) {
        unsafe {
            SCREEN_MANAGER.lock().write_char(c);
        }
    }
    
    fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }
    
    fn newline(&mut self) {
        self.write_char('\n');
    }
    
    fn clear_screen(&mut self) {
        unsafe {
            SCREEN_MANAGER.lock().clear();
        }
    }
    
    fn set_cursor(&mut self, row: usize, col: usize) {
        unsafe {
            SCREEN_MANAGER.lock().set_cursor(row, col);
        }
    }
    
    fn get_cursor(&self) -> (usize, usize) {
        unsafe {
            SCREEN_MANAGER.lock().get_cursor()
        }
    }
}

/// State machine implementation for kernel_1
pub struct Kernel1StateMachine {
    state: ShellState,
    command_buffer: CommandBuffer,
    prompt: &'static str,
}

impl Kernel1StateMachine {
    pub const fn new() -> Self {
        Self {
            state: ShellState::Uninitialized,
            command_buffer: CommandBuffer::new(),
            prompt: "kernel1> ",
        }
    }
}

impl ShellStateMachine for Kernel1StateMachine {
    fn process_input(&mut self, input: char) -> ShellAction {
        match self.state {
            ShellState::Uninitialized => {
                self.state = ShellState::Waiting;
                ShellAction::DisplayPrompt
            }
            ShellState::Waiting => {
                match input {
                    '\n' => {
                        if !self.command_buffer.is_empty() {
                            self.state = ShellState::Ready;
                            ShellAction::ExecuteCommand
                        } else {
                            ShellAction::DisplayPrompt
                        }
                    }
                    '\x08' => {
                        // Backspace
                        if self.command_buffer.pop().is_some() {
                            ShellAction::ClearLine
                        } else {
                            ShellAction::Bell
                        }
                    }
                    c if c.is_ascii_graphic() || c == ' ' => {
                        if self.command_buffer.push(c) {
                            ShellAction::Continue
                        } else {
                            ShellAction::Bell
                        }
                    }
                    _ => ShellAction::Continue,
                }
            }
            ShellState::Ready => {
                self.state = ShellState::Waiting;
                self.command_buffer.clear();
                ShellAction::DisplayPrompt
            }
            ShellState::Executing => {
                self.state = ShellState::Waiting;
                ShellAction::Continue
            }
        }
    }
    
    fn get_prompt(&self) -> &str {
        self.prompt
    }
    
    fn get_state(&self) -> ShellState {
        self.state
    }
    
    fn reset(&mut self) {
        self.state = ShellState::Uninitialized;
        self.command_buffer.clear();
    }
    
    fn get_command_buffer(&self) -> &str {
        self.command_buffer.as_str()
    }
    
    fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }
}

/// Basic command executor for kernel_1
pub struct Kernel1CommandExecutor {
    commands: [&'static str; 4],
}

impl Kernel1CommandExecutor {
    pub const fn new() -> Self {
        Self {
            commands: ["help", "clear", "halt", "reboot"],
        }
    }
    
    fn execute_help(&mut self, output: &mut dyn OutputProvider) {
        output.write_str("Available commands:\n");
        for cmd in &self.commands {
            output.write_str("  ");
            output.write_str(cmd);
            output.newline();
        }
    }
    
    fn execute_clear(&mut self, output: &mut dyn OutputProvider) {
        output.clear_screen();
    }
    
    fn execute_halt(&mut self, output: &mut dyn OutputProvider) {
        output.write_str("System halt requested\n");
        // In a real implementation, this would halt the system
        // For now, we just display a message
    }
    
    fn execute_reboot(&mut self, output: &mut dyn OutputProvider) {
        output.write_str("System reboot requested\n");
        // In a real implementation, this would reboot the system
        // For now, we just display a message
    }
}

impl CommandExecutor for Kernel1CommandExecutor {
    fn execute(&mut self, command: &str, _args: &[&str]) -> CommandResult {
        match command {
            "help" => {
                // Note: We can't call execute_help here because we don't have access to output
                // This will be handled by the shell implementation
                CommandResult::Success
            }
            "clear" => CommandResult::Success,
            "halt" => CommandResult::Success,
            "reboot" => CommandResult::Success,
            "" => CommandResult::Success,
            _ => CommandResult::InvalidCommand,
        }
    }
    
    fn get_commands(&self) -> &[&str] {
        &self.commands
    }
    
    fn get_help(&self, command: &str) -> Option<&str> {
        match command {
            "help" => Some("Display this help message"),
            "clear" => Some("Clear the screen"),
            "halt" => Some("Halt the system"),
            "reboot" => Some("Reboot the system"),
            _ => None,
        }
    }
}

/// Complete shell implementation for kernel_1
pub struct Kernel1Shell {
    input: Kernel1InputAdapter,
    output: Kernel1OutputAdapter,
    executor: Kernel1CommandExecutor,
    state_machine: Kernel1StateMachine,
    initialized: bool,
}

impl Kernel1Shell {
    pub const fn new() -> Self {
        Self {
            input: Kernel1InputAdapter::new(),
            output: Kernel1OutputAdapter::new(),
            executor: Kernel1CommandExecutor::new(),
            state_machine: Kernel1StateMachine::new(),
            initialized: false,
        }
    }
    
    fn display_prompt(&mut self) {
        let prompt = self.state_machine.get_prompt();
        self.output.write_str(prompt);
        
        // Display current command buffer
        let buffer = self.state_machine.get_command_buffer();
        self.output.write_str(buffer);
    }
    
    fn clear_line(&mut self) {
        // Move cursor to beginning of line and clear
        self.output.write_char('\r');
        let prompt = self.state_machine.get_prompt();
        self.output.write_str(prompt);
        let buffer = self.state_machine.get_command_buffer();
        self.output.write_str(buffer);
    }
    
    fn execute_current_command(&mut self) {
        let command_line = self.state_machine.get_command_buffer();
        let (command, args, arg_count) = parse_command_line(command_line);
        
        match command {
            "help" => {
                self.output.newline();
                self.output.write_str("Available commands:\n");
                for cmd in self.executor.get_commands() {
                    self.output.write_str("  ");
                    self.output.write_str(cmd);
                    if let Some(help) = self.executor.get_help(cmd) {
                        self.output.write_str(" - ");
                        self.output.write_str(help);
                    }
                    self.output.newline();
                }
            }
            "clear" => {
                self.output.clear_screen();
            }
            "halt" => {
                self.output.newline();
                self.output.write_str("System halt requested\n");
            }
            "reboot" => {
                self.output.newline();
                self.output.write_str("System reboot requested\n");
            }
            "" => {
                self.output.newline();
            }
            _ => {
                self.output.newline();
                self.output.write_str("Unknown command: ");
                self.output.write_str(command);
                self.output.write_str("\nType 'help' for available commands\n");
            }
        }
    }
}

impl Shell for Kernel1Shell {
    type Input = Kernel1InputAdapter;
    type Output = Kernel1OutputAdapter;
    type Executor = Kernel1CommandExecutor;
    type StateMachine = Kernel1StateMachine;
    
    fn input_mut(&mut self) -> &mut Self::Input {
        &mut self.input
    }
    
    fn output_mut(&mut self) -> &mut Self::Output {
        &mut self.output
    }
    
    fn executor_mut(&mut self) -> &mut Self::Executor {
        &mut self.executor
    }
    
    fn state_machine_mut(&mut self) -> &mut Self::StateMachine {
        &mut self.state_machine
    }
    
    fn process_iteration(&mut self) -> bool {
        if !self.initialized {
            self.initialize();
            return true;
        }
        
        if let Some(input_char) = self.input.get_next_char() {
            let action = self.state_machine.process_input(input_char);
            
            match action {
                ShellAction::Continue => {
                    self.output.write_char(input_char);
                }
                ShellAction::ExecuteCommand => {
                    self.execute_current_command();
                }
                ShellAction::DisplayPrompt => {
                    self.display_prompt();
                }
                ShellAction::ClearLine => {
                    self.clear_line();
                }
                ShellAction::Bell => {
                    // Ignore bell for now
                }
            }
            
            true
        } else {
            false
        }
    }
    
    fn initialize(&mut self) {
        self.output.write_str("Kernel Shell v1.0\n");
        self.output.write_str("Type 'help' for available commands\n");
        self.display_prompt();
        self.initialized = true;
    }
}