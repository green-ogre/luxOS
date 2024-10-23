// use core::{
//     cell::UnsafeCell,
//     mem::MaybeUninit,
//     sync::atomic::{AtomicBool, AtomicUsize, Ordering},
// };
//
// use alloc::{boxed::Box, sync::Arc, vec::Vec};
//
// struct Channel<T: Send> {
//     read: AtomicUsize,
//     write: AtomicUsize,
//     allocating: AtomicBool,
//     buf: Vec<UnsafeCell<MaybeUninit<T>>>,
// }
//
// impl<T: Send> Channel<T> {
//     pub fn write(&self, elem: T) {
//         let write = self.write.load(Ordering::SeqCst);
//
//         loop {
//             if write == self.buf.len() {
//                 match self.allocating.compare_exchange_weak(
//                     false,
//                     true,
//                     Ordering::AcqRel,
//                     Ordering::Relaxed,
//                 ) {
//                     Ok(_) => {
//                         // let read = self.read.load(Ordering::Acquire);
//                     }
//                     Err(_) => {
//                         // TODO: wait for allocation to finish, want to see if this runs.
//                         unimplemented!();
//                     }
//                 }
//             } else {
//                 break;
//             }
//         }
//
//         // if (read != write && write == read.saturating_sub(1))
//         //     || read == 0 && write == self.buf.len().saturating_sub(1)
//         // {
//         //     // TODO: can this cause repeated reads?
//         //     self.read.store(new_idx(read), Ordering::Release);
//         // }
//         //
//         // let index = new_idx(write);
//         // match self
//         //     .write
//         //     .compare_exchange_weak(write, index, Ordering::AcqRel, Ordering::Relaxed)
//         // {
//         //     Ok(_) => {}
//         //     Err(_) => {
//         //         // TODO: releaxed read, then retry, honestly just want to see if this ever runs on
//         //         // my machine in practice.
//         //         //
//         //         // write = self.write.load(Ordering::Relaxed);
//         //         // index = new_idx(write);
//         //         unimplemented!();
//         //     }
//         // }
//         //
//         // unsafe { *self.buf[write].get() = MaybeUninit::new(elem) };
//     }
//
//     pub fn read(&self) -> Option<T> {
//         let new_idx = |idx: usize| -> usize {
//             if idx >= self.buf.len().saturating_sub(1) {
//                 0
//             } else {
//                 idx + 1
//             }
//         };
//
//         let write = self.write.load(Ordering::SeqCst);
//         let read = self.read.load(Ordering::SeqCst);
//
//         if read != write {
//             let index = new_idx(read);
//             match self
//                 .read
//                 .compare_exchange_weak(read, index, Ordering::AcqRel, Ordering::Relaxed)
//             {
//                 Ok(_) => {}
//                 Err(_) => {
//                     // TODO: releaxed read, then retry, honestly just want to see if this ever runs on
//                     // my machine in practice.
//                     //
//                     // read = self.read.load(Ordering::Relaxed);
//                     // index = new_idx(read);
//                     unimplemented!();
//                 }
//             }
//
//             let mut data = MaybeUninit::uninit();
//
//             unsafe {
//                 core::mem::swap(self.buf[read].get().as_mut().unwrap(), &mut data);
//                 Some(data.assume_init())
//             }
//         } else {
//             None
//         }
//     }
// }
//
// impl<T: Send> Drop for Channel<T> {
//     fn drop(&mut self) {
//         if core::mem::needs_drop::<T>() {
//             while let Some(_) = self.read() {}
//         }
//     }
// }
//
// pub struct Sender<T: Send> {
//     data: Arc<Channel<T>>,
// }
//
// impl<T: Send> Sender<T> {
//     pub fn send(&self, data: T) {
//         self.data.write(data);
//     }
// }
//
// pub struct Receiver<T: Send> {
//     data: Arc<Channel<T>>,
// }
//
// impl<T: Send> Receiver<T> {
//     pub fn receive(&self) -> Option<T> {
//         self.data.read()
//     }
// }
