use core::fmt::Write;
use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;
use crate::command::CommandHandler;
use crate::kspin_lock::kspin_lock::KSpinLock;
use crate::printk;

// Global command handler for screen 1
static COMMAND_HANDLER: KSpinLock<CommandHandler> = KSpinLock::new(CommandHandler::new());

/// Initialize the command handler and user terminal
pub fn init_command_handler() {
    let mut manager = screen_manager().lock();
    
    // Write welcome message to screen 2 (user command screen)
    if let Some(screen) = manager.get_screen_mut(2) {
        let mut writer = Writer::new(screen);
        write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
        write!(writer, "\n").unwrap();
        write!(writer, "Type 'help' for available commands.\n").unwrap();
        write!(writer, "> ").unwrap();
        
        // Get current cursor position for prompt
        let prompt_row = screen.row_position;
        let prompt_col = screen.column_position;
        
        // Flush to physical screen and update cursor if screen 2 is active
        if manager.get_active_screen_id() == 2 {
            manager.flush_to_physical();
            manager.update_cursor();
        }
        
        // Release manager lock before accessing command handler
        drop(manager);
        
        // Set prompt position in command handler
        let mut cmd_handler = COMMAND_HANDLER.lock();
        cmd_handler.set_prompt_position(prompt_row, prompt_col);
        drop(cmd_handler);
    }
    
    // Add informational message to kernel log
    printk!(LogLevel::Info, "Command handler initialized.\n");
    printk!(LogLevel::Info, "User interface ready on Screen 2 - switch with Ctrl+Right\n");
}

/// Get reference to the global command handler
pub fn command_handler() -> &'static KSpinLock<CommandHandler> {
    &COMMAND_HANDLER
}