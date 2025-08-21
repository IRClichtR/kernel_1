use core::fmt::Write;
use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;
use crate::command::CommandHandler;
use crate::kspin_lock::kspin_lock::KSpinLock;
use crate::printk;

static mut COMMAND_HANDLER: KSpinLock<CommandHandler> = KSpinLock::new(CommandHandler::new());

pub fn init_command_handler() {
    let mut manager = screen_manager().lock();
    
    if let Some(screen) = manager.get_screen_mut(2) {
        let mut writer = Writer::new(screen);
        write!(writer, "#                             Welcome to the User Terminal                     #\n").unwrap();
        write!(writer, "\n").unwrap();
        write!(writer, "Type 'help' for available commands.\n").unwrap();
        write!(writer, "> ").unwrap();
        
        let prompt_row = screen.row_position;
        let prompt_col = screen.column_position;
        
        if manager.get_active_screen_id() == 2 {
            manager.flush_to_physical();
            manager.update_cursor();
        }
        
        drop(manager);
        
        unsafe {
            let cmd_handler_ptr = &raw mut COMMAND_HANDLER;
            let mut cmd_handler = (*cmd_handler_ptr).lock();
            cmd_handler.set_prompt_position(prompt_row, prompt_col);
            drop(cmd_handler);
        }
    }
    
    printk!(LogLevel::Info, "Command handler initialized.\n");
    printk!(LogLevel::Info, "User interface ready on Screen 2 - switch with Ctrl+Right\n");
}

pub fn command_handler() -> &'static KSpinLock<CommandHandler> {
    unsafe { 
        let cmd_handler_ptr = &raw const COMMAND_HANDLER;
        &*cmd_handler_ptr
    }
}