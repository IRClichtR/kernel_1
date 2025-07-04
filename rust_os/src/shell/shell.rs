// src/shell/shell.rs
// Updated shell implementation using the abstraction layers

#![no_std]

use super::interfaces::*;
use super::adapters::*;

/// Global shell instance
static mut SHELL: Kernel1Shell = Kernel1Shell::new();

/// Initialize the shell system
pub fn init_shell() {
    unsafe {
        SHELL.initialize();
    }
}

/// Process shell input/output - should be called from main loop
pub fn process_shell() {
    unsafe {
        SHELL.process_iteration();
    }
}

/// Get mutable reference to the shell (unsafe - single threaded kernel)
pub fn get_shell_mut() -> &'static mut Kernel1Shell {
    unsafe { &mut SHELL }
}

/// Execute a command directly (for testing or internal use)
pub fn execute_command(command: &str) -> CommandResult {
    let (cmd, args, _arg_count) = parse_command_line(command);
    unsafe {
        SHELL.executor_mut().execute(cmd, &args[..])
    }
}

/// Write to shell output
pub fn shell_write(s: &str) {
    unsafe {
        SHELL.output_mut().write_str(s);
    }
}

/// Write a single character to shell output
pub fn shell_write_char(c: char) {
    unsafe {
        SHELL.output_mut().write_char(c);
    }
}

/// Clear the shell screen
pub fn shell_clear() {
    unsafe {
        SHELL.output_mut().clear_screen();
    }
}

/// Reset the shell to initial state
pub fn shell_reset() {
    unsafe {
        SHELL.state_machine_mut().reset();
        SHELL.initialize();
    }
}

/// Get current shell state
pub fn get_shell_state() -> ShellState {
    unsafe {
        SHELL.state_machine_mut().get_state()
    }
}

/// Check if shell has input available
pub fn shell_has_input() -> bool {
    unsafe {
        SHELL.input_mut().has_input()
    }
}

/// Get shell prompt
pub fn get_shell_prompt() -> &'static str {
    unsafe {
        SHELL.state_machine_mut().get_prompt()
    }
}

/// Advanced shell operations for extension
pub mod advanced {
    use super::*;
    
    /// Register a custom command (for future extensibility)
    pub fn register_command(_name: &str, _help: &str) -> Result<(), &'static str> {
        // This would be implemented when we add dynamic command registration
        Err("Dynamic command registration not yet implemented")
    }
    
    /// Get detailed shell statistics
    pub fn get_shell_stats() -> ShellStats {
        ShellStats {
            commands_executed: 0, // Would be tracked in implementation
            state: get_shell_state(),
            buffer_usage: 0,      // Would be calculated from buffer
        }
    }
}

/// Shell statistics structure
#[derive(Debug, Clone, Copy)]
pub struct ShellStats {
    pub commands_executed: usize,
    pub state: ShellState,
    pub buffer_usage: usize,
}

/// Compatibility layer for existing kernel_1 code
pub mod compat {
    use super::*;
    
    /// Legacy shell processing function - maintains compatibility
    pub fn handle_shell_input() {
        process_shell();
    }
    
    /// Legacy command execution
    pub fn run_command(cmd: &str) -> bool {
        match execute_command(cmd) {
            CommandResult::Success => true,
            _ => false,
        }
    }
    
    /// Legacy screen operations
    pub fn print_to_shell(s: &str) {
        shell_write(s);
    }
    
    /// Legacy screen clearing
    pub fn clear_shell_screen() {
        shell_clear();
    }
}

// Unit tests for the shell system (when testing is available)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_parsing() {
        let (cmd, args, count) = parse_command_line("help arg1 arg2");
        assert_eq!(cmd, "help");
        assert_eq!(args[0], "arg1");
        assert_eq!(args[1], "arg2");
        assert_eq!(count, 2);
    }
    
    #[test]
    fn test_command_buffer() {
        let mut buffer = CommandBuffer::new();
        assert!(buffer.is_empty());
        
        buffer.push('h');
        buffer.push('i');
        assert_eq!(buffer.as_str(), "hi");
        
        buffer.pop();
        assert_eq!(buffer.as_str(), "h");
        
        buffer.clear();
        assert!(buffer.is_empty());
    }
    
    #[test]
    fn test_shell_state_transitions() {
        let mut state_machine = Kernel1StateMachine::new();
        
        // Test initialization
        let action = state_machine.process_input('h');
        assert_eq!(action, ShellAction::DisplayPrompt);
        
        // Test command input
        let action = state_machine.process_input('i');
        assert_eq!(action, ShellAction::Continue);
        
        // Test command execution
        let action = state_machine.process_input('\n');
        assert_eq!(action, ShellAction::ExecuteCommand);
    }
}