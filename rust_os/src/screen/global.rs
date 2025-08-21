use core::mem::MaybeUninit;
use core::ptr::addr_of;
use crate::printk;
use super::manager::ScreenManager;
use crate::kspin_lock::kspin_lock::KSpinLock;

static mut SCREEN_MANAGER: MaybeUninit<KSpinLock<ScreenManager>> = MaybeUninit::uninit();

pub fn init_screen_manager() {
    unsafe {
        SCREEN_MANAGER = MaybeUninit::new(KSpinLock::new(ScreenManager::new()));
    }
    
    {
        let mut manager = screen_manager().lock();
        manager.clear_screen(1);
        manager.clear_screen(2);
        manager.flush_to_physical();
        manager.update_cursor();
    }
    
    printk!(LogLevel::Info, "Screen manager initialized.\n");
    printk!(LogLevel::Info, "=== Dual Screen System ===\n");
    printk!(LogLevel::Info, "Screen 1: Kernel messages and system logs (current)\n");
    printk!(LogLevel::Info, "Screen 2: User command interface\n");
    printk!(LogLevel::Info, "Use Ctrl+Left/Right arrows to switch between screens\n");
    printk!(LogLevel::Info, "=============================\n");
}

pub fn screen_manager() -> &'static KSpinLock<ScreenManager> {
    unsafe { 
        let ptr = addr_of!(SCREEN_MANAGER);
        (*ptr).assume_init_ref()
    }
}