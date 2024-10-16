use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub fn new(inner: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(inner),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
            .is_err()
        {}
        debug_assert!(self.lock.load(Ordering::Acquire));
        SpinLockGuard::new(&self.lock, self.data.get())
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a AtomicBool,
    data: *mut T,
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref().unwrap() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.data.as_mut().unwrap() }
    }
}

impl<'a, T> SpinLockGuard<'a, T> {
    pub fn new(lock: &'a AtomicBool, data: *mut T) -> Self {
        Self { lock, data }
    }
}
