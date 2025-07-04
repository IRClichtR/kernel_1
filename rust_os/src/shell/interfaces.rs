// src/shell/interfaces.rs
// Core abstraction traits for shell I/O and command handling

#![no_std]

/// Result type for command execution
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandResult {
    Success,
    InvalidCommand,
    InvalidArguments,
    SystemError,
}

/// Actions that the shell state machine can trigger
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellAction {
    Continue,
    ExecuteCommand,
    DisplayPrompt,
    ClearLine,
    Bell,
}

/// Shell state machine states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellState {
    Uninitialized,
    Waiting,
    Ready,
    Executing,
}

/// Input provider abstraction - converts different input sources to character stream
pub trait InputProvider {
    /// Get the next character from input if available
    fn get_next_char(&mut self) -> Option<char>;
    
    /// Check if input is available without consuming it
    fn has_input(&self) -> bool;
    
    /// Reset input state (clear buffers, etc.)
    fn reset(&mut self);
}

/// Output provider abstraction - handles all output operations
pub trait OutputProvider {
    /// Write a single character
    fn write_char(&mut self, c: char);
    
    /// Write a string
    fn write_str(&mut self, s: &str);
    
    /// Write a newline
    fn newline(&mut self);
    
    /// Clear the screen
    fn clear_screen(&mut self);
    
    /// Set cursor position if supported
    fn set_cursor(&mut self, row: usize, col: usize);
    
    /// Get current cursor position if supported
    fn get_cursor(&self) -> (usize, usize);
}

/// Command executor abstraction - handles command parsing and execution
pub trait CommandExecutor {
    /// Execute a command with arguments
    fn execute(&mut self, command: &str, args: &[&str]) -> CommandResult;
    
    /// Get list of available commands
    fn get_commands(&self) -> &[&str];
    
    /// Get help text for a specific command
    fn get_help(&self, command: &str) -> Option<&str>;
}

/// Shell state machine abstraction
pub trait ShellStateMachine {
    /// Process input character and return action to take
    fn process_input(&mut self, input: char) -> ShellAction;
    
    /// Get current prompt string
    fn get_prompt(&self) -> &str;
    
    /// Get current state
    fn get_state(&self) -> ShellState;
    
    /// Reset shell to initial state
    fn reset(&mut self);
    
    /// Get current command buffer
    fn get_command_buffer(&self) -> &str;
    
    /// Clear command buffer
    fn clear_command_buffer(&mut self);
}

/// Complete shell interface that combines all components
pub trait Shell {
    type Input: InputProvider;
    type Output: OutputProvider;
    type Executor: CommandExecutor;
    type StateMachine: ShellStateMachine;
    
    /// Get mutable reference to input provider
    fn input_mut(&mut self) -> &mut Self::Input;
    
    /// Get mutable reference to output provider
    fn output_mut(&mut self) -> &mut Self::Output;
    
    /// Get mutable reference to command executor
    fn executor_mut(&mut self) -> &mut Self::Executor;
    
    /// Get mutable reference to state machine
    fn state_machine_mut(&mut self) -> &mut Self::StateMachine;
    
    /// Process one iteration of shell loop
    fn process_iteration(&mut self) -> bool;
    
    /// Initialize shell
    fn initialize(&mut self);
}

/// Maximum command buffer size
pub const MAX_COMMAND_BUFFER: usize = 256;

/// Maximum number of command arguments
pub const MAX_ARGS: usize = 16;

/// Command buffer for storing input
pub struct CommandBuffer {
    buffer: [u8; MAX_COMMAND_BUFFER],
    length: usize,
}

impl CommandBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [0; MAX_COMMAND_BUFFER],
            length: 0,
        }
    }
    
    pub fn push(&mut self, c: char) -> bool {
        if self.length >= MAX_COMMAND_BUFFER - 1 {
            return false;
        }
        
        self.buffer[self.length] = c as u8;
        self.length += 1;
        true
    }
    
    pub fn pop(&mut self) -> Option<char> {
        if self.length == 0 {
            return None;
        }
        
        self.length -= 1;
        Some(self.buffer[self.length] as char)
    }
    
    pub fn clear(&mut self) {
        self.length = 0;
    }
    
    pub fn as_str(&self) -> &str {
        // Safety: we only push valid ASCII characters
        unsafe { core::str::from_utf8_unchecked(&self.buffer[..self.length]) }
    }
    
    pub fn len(&self) -> usize {
        self.length
    }
    
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

/// Parse command line into command and arguments
pub fn parse_command_line(line: &str) -> (&str, [&str; MAX_ARGS], usize) {
    let mut args = [""; MAX_ARGS];
    let mut arg_count = 0;
    let mut parts = line.split_whitespace();
    
    let command = parts.next().unwrap_or("");
    
    for part in parts {
        if arg_count >= MAX_ARGS {
            break;
        }
        args[arg_count] = part;
        arg_count += 1;
    }
    
    (command, args, arg_count)
}