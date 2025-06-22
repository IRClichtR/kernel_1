use core::mem::MaybeUninit;
use super::manager::ScreenManager;
use crate::kspin_lock::kspin_lock::KSpinLock;

static mut SCREEN_MANAGER: MaybeUninit<KSpinLock<ScreenManager>> = MaybeUninit::uninit();

pub fn init_screen_manager() {
    unsafe {
        SCREEN_MANAGER = MaybeUninit::new(KSpinLock::new(ScreenManager::new()));
    }
}

pub fn screen_manager() -> &'static KSpinLock<ScreenManager> {
    unsafe { SCREEN_MANAGER.assume_init_ref() }
} 