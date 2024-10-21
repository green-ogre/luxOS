use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Queue of `0..CAP - 1` elements.
///
/// Any subsequent writes past `CAP - 1` overwrite previous values.
pub struct CircularBuffer<T, const CAP: usize> {
    read: AtomicUsize,
    write: AtomicUsize,
    buf: [UnsafeCell<MaybeUninit<T>>; CAP],
}

unsafe impl<T, const CAP: usize> Sync for CircularBuffer<T, CAP> {}

impl<T, const CAP: usize> CircularBuffer<T, CAP> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
            buf: MakeCircularBuffer::UNINIT_ARRAY,
        }
    }

    pub fn write(&self, elem: T) {
        let new_idx = |idx: usize| -> usize {
            if idx >= CAP.saturating_sub(1) {
                0
            } else {
                idx + 1
            }
        };

        let mut write = self.write.load(Ordering::Relaxed);

        // Need to see if write is the index behind read, and if so, also increment the read.
        let read = self.read.load(Ordering::Relaxed);
        if (read != write && write == read.saturating_sub(1))
            || read == 0 && write == CAP.saturating_sub(1)
        {
            // TODO: can this cause repeated reads?
            self.read.store(new_idx(read), Ordering::Release);
        }

        let mut index = new_idx(write);
        while self
            .write
            .compare_exchange_weak(write, index, Ordering::SeqCst, Ordering::Relaxed)
            .is_err()
        {
            write = self.write.load(Ordering::Relaxed);
            index = new_idx(write);
        }

        unsafe { *self.buf[write].get() = MaybeUninit::new(elem) };
    }

    pub fn read(&self) -> Option<T> {
        let new_idx = |idx: usize| -> usize {
            if idx >= CAP.saturating_sub(1) {
                0
            } else {
                idx + 1
            }
        };

        let write = self.write.load(Ordering::Relaxed);
        let mut read = self.read.load(Ordering::Relaxed);

        if read != write {
            let mut index = new_idx(read);
            while self
                .read
                .compare_exchange_weak(read, index, Ordering::SeqCst, Ordering::Relaxed)
                .is_err()
            {
                read = self.read.load(Ordering::Relaxed);
                index = new_idx(read);
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

struct MakeCircularBuffer<T, const CAP: usize> {
    _phantom: PhantomData<T>,
}

impl<T, const CAP: usize> MakeCircularBuffer<T, CAP> {
    #[allow(clippy::declare_interior_mutable_const)]
    const UNINIT: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());
    #[allow(clippy::declare_interior_mutable_const)]
    const UNINIT_ARRAY: [UnsafeCell<MaybeUninit<T>>; CAP] = [Self::UNINIT; CAP];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_case;

    test_case!(circular_buffer, {
        let buf = CircularBuffer::<u32, 4>::new();
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
