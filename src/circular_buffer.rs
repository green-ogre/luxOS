use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub struct CircularBuffer<T> {
    read: AtomicUsize,
    write: AtomicUsize,
    buf: Box<[UnsafeCell<MaybeUninit<T>>]>,
}

unsafe impl<T> Sync for CircularBuffer<T> {}

impl<T> CircularBuffer<T> {
    pub fn new(cap: usize) -> Self {
        Self {
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
            buf: (0..cap)
                .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
                .collect(),
        }
    }

    pub fn write(&self, elem: T) {
        let new_idx = |idx: usize| -> usize {
            if idx >= self.buf.len().saturating_sub(1) {
                0
            } else {
                idx + 1
            }
        };

        let write = self.write.load(Ordering::SeqCst);

        let read = self.read.load(Ordering::Acquire);
        if (read != write && write == read.saturating_sub(1))
            || read == 0 && write == self.buf.len().saturating_sub(1)
        {
            // TODO: can this cause repeated reads?
            self.read.store(new_idx(read), Ordering::Release);
        }

        let index = new_idx(write);
        match self
            .write
            .compare_exchange_weak(write, index, Ordering::AcqRel, Ordering::Relaxed)
        {
            Ok(_) => {}
            Err(_) => {
                // TODO: releaxed read, then retry, honestly just want to see if this ever runs on
                // my machine in practice.
                //
                // write = self.write.load(Ordering::Relaxed);
                // index = new_idx(write);
                unimplemented!();
            }
        }

        unsafe { *self.buf[write].get() = MaybeUninit::new(elem) };
    }

    pub fn read(&self) -> Option<T> {
        let new_idx = |idx: usize| -> usize {
            if idx >= self.buf.len().saturating_sub(1) {
                0
            } else {
                idx + 1
            }
        };

        let write = self.write.load(Ordering::SeqCst);
        let read = self.read.load(Ordering::SeqCst);

        if read != write {
            let index = new_idx(read);
            match self
                .read
                .compare_exchange_weak(read, index, Ordering::AcqRel, Ordering::Relaxed)
            {
                Ok(_) => {}
                Err(_) => {
                    // TODO: releaxed read, then retry, honestly just want to see if this ever runs on
                    // my machine in practice.
                    //
                    // read = self.read.load(Ordering::Relaxed);
                    // index = new_idx(read);
                    unimplemented!();
                }
            }

            let mut data = MaybeUninit::uninit();

            unsafe {
                core::mem::swap(self.buf[read].get().as_mut().unwrap(), &mut data);
                Some(data.assume_init())
            }
        } else {
            None
        }
    }
}

impl<T> Drop for CircularBuffer<T> {
    fn drop(&mut self) {
        if core::mem::needs_drop::<T>() {
            while self.read().is_some() {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_case;

    test_case!(circular_buffer, {
        let buf = CircularBuffer::new(4);
        test_assert_eq!(None, buf.read());

        buf.write(1);
        buf.write(2);
        buf.write(3);

        test_assert_eq!(Some(1), buf.read());
        test_assert_eq!(Some(2), buf.read());
        test_assert_eq!(Some(3), buf.read());
        test_assert_eq!(None, buf.read());

        buf.write(1);
        test_assert_eq!(Some(1), buf.read());
        buf.write(1);
        test_assert_eq!(Some(1), buf.read());
        buf.write(1);
        test_assert_eq!(Some(1), buf.read());
        buf.write(1);
        test_assert_eq!(Some(1), buf.read());
        buf.write(1);
        test_assert_eq!(Some(1), buf.read());
        buf.write(1);
        test_assert_eq!(Some(1), buf.read());
        buf.write(1);
        test_assert_eq!(Some(1), buf.read());
        buf.write(1);
        test_assert_eq!(Some(1), buf.read());

        buf.write(2);
        buf.write(3);
        buf.write(4);
        buf.write(5);
        buf.write(6);

        test_assert_eq!(Some(4), buf.read());
        test_assert_eq!(Some(5), buf.read());
        test_assert_eq!(Some(6), buf.read());
        test_assert_eq!(None, buf.read());
        test_assert_eq!(None, buf.read());
        test_assert_eq!(None, buf.read());
        test_assert_eq!(None, buf.read());
    });
}
