use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct KSpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

pub struct KSpinLockGuard<'a, T> {
    lock: &'a KSpinLock<T>,
}

unsafe impl<T: Send> Send for KSpinLock<T> {}
unsafe impl<T: Send> Sync for KSpinLock<T> {}

impl<T> KSpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> KSpinLockGuard<'_, T> {
        while self.locked.compare_exchange_weak(
            false, true, 
            Ordering::Acquire, 
            Ordering::Relaxed
        ).is_err() {
            core::hint::spin_loop();
        }

        KSpinLockGuard { lock: self }
    }
}

impl<T> Deref for KSpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for KSpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for KSpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
} 