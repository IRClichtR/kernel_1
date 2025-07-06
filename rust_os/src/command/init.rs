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
    if let Some(_screen_id) = manager.create_screen() {            
        if manager.switch_screen(1) {
            if let Some(screen) = &mut manager.screens[1] {
                let mut writer = Writer::new(screen);
                write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
                write!(writer, "\n").unwrap();
                write!(writer, "Type 'help' for available commands.\n").unwrap();
                write!(writer, "> ").unwrap();
                
                // Get current cursor position for prompt - using correct field names
                let prompt_row = screen.row_position;
                let prompt_col = screen.column_position;
                
                // Release manager lock before accessing command handler
                drop(manager);
                
                // Set prompt position in command handler
                let mut cmd_handler = COMMAND_HANDLER.lock();
                cmd_handler.set_prompt_position(prompt_row, prompt_col);
                drop(cmd_handler);
                
                // Re-acquire manager lock for screen switch
                let mut manager = screen_manager().lock();
                manager.switch_screen(0);
            }
        }
    }
}

/// Get reference to the global command handler
pub fn command_handler() -> &'static KSpinLock<CommandHandler> {
    &COMMAND_HANDLER
}