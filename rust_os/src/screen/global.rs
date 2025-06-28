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
    printk!(LogLevel::Info, "Screen manager initialized.\n");
}

pub fn screen_manager() -> &'static KSpinLock<ScreenManager> {
    unsafe { 
        let ptr = addr_of!(SCREEN_MANAGER);
        (*ptr).assume_init_ref()
    }
}