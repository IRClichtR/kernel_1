use core::fmt::Write;
use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;
use crate::command::CommandHandler;
use crate::kspin_lock::kspin_lock::KSpinLock;

// Global command handler for screen 1
static COMMAND_HANDLER: KSpinLock<CommandHandler> = KSpinLock::new(CommandHandler::new());

/// Initialize the command handler and user terminal
pub fn init_command_handler() {
    let mut manager = screen_manager().lock();
    
    // Write welcome message to the screen
    let mut writer = Writer::new(&mut manager.screen);
    write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
    write!(writer, "\n").unwrap();
    write!(writer, "Type 'help' for available commands.\n").unwrap();
    write!(writer, "> ").unwrap();
    
    // Flush to physical screen and update cursor
    manager.flush_to_physical();
    manager.update_cursor();
    
    // Get current cursor position for prompt
    let prompt_row = manager.screen.row_position;
    let prompt_col = manager.screen.column_position;
    
    // Release manager lock before accessing command handler
    drop(manager);
    
    // Set prompt position in command handler
    let mut cmd_handler = COMMAND_HANDLER.lock();
    cmd_handler.set_prompt_position(prompt_row, prompt_col);
    drop(cmd_handler);
}

/// Get reference to the global command handler
pub fn command_handler() -> &'static KSpinLock<CommandHandler> {
    &COMMAND_HANDLER
}