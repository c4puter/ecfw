/*
 * The MIT License (MIT)
 * Copyright (c) 2016 Chris Pavlina
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
 * OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
 * OR OTHER DEALINGS IN THE SOFTWARE.
 */

/*!
Lock-free threadsafe queue.
*/

use rustsys::freertos;
use core::sync::atomic::*;
use core::cell::*;
use core::mem;
use core::slice;
use alloc::boxed::*;
use alloc::heap;

pub struct Queue<'a, T> where T: 'a + Send + Copy + Sized {
    data: &'a [Cell<Option<T>>],
    _owned_data: Option<Box<[Cell<Option<T>>]>>,
    iread: AtomicUsize,
    imaxread: AtomicUsize,
    iwrite: AtomicUsize,
    flushing: AtomicBool,
}

macro_rules! queue_static_init {
    ( $n:expr ) => (
        repeat_expr!{$n { ::core::cell::Cell::new(None) } }
    )
}

#[macro_export]
macro_rules! queue_static_new {
    ( $name:ident, $ty:ty, $n:expr ) => (

        static $name: queue::Queue<'static, $ty> = unsafe {
            queue::Queue::new_static(
                {static mut BACKING: [::core::cell::Cell<Option<u8>>; $n]
                                     = queue_static_init!($n); &BACKING}
            )
        };
    )
}

unsafe impl<'a, T> Sync for Queue<'a, T> where T: 'a + Send + Copy + Sized {}

impl<'a, T> Queue<'a, T> where T: 'a + Send + Copy + Sized {
    /// Create a new queue using a static backing array. This is unsafe because
    /// it "takes" the array; nothing else should ever touch it.
    ///
    /// Don't use this function. See queue_static_new!.
    pub const unsafe fn new_static(buffer: &'static [Cell<Option<T>>]) -> Queue<'a, T> {
        Queue {
            data: buffer,
            _owned_data: None,
            iread: ATOMIC_USIZE_INIT,
            imaxread: ATOMIC_USIZE_INIT,
            iwrite: ATOMIC_USIZE_INIT,
            flushing: ATOMIC_BOOL_INIT,
        }
    }

    pub fn new_dynamic(sz: usize) -> Queue<'a, T> {
        let buffer = unsafe{heap::allocate(sz * mem::size_of::<Cell<Option<T>>>(), 4)};
        let as_slice = unsafe{slice::from_raw_parts_mut(buffer as *mut Cell<Option<T>>, sz)};
        for i in 0..sz {
            as_slice[i] = Cell::new(None);
        }
        let as_box = unsafe{Box::from_raw(as_slice)};

        Queue {
            data: as_slice,
            _owned_data: Some(as_box),
            iread: ATOMIC_USIZE_INIT,
            imaxread: ATOMIC_USIZE_INIT,
            iwrite: ATOMIC_USIZE_INIT,
            flushing: ATOMIC_BOOL_INIT,
        }
    }

    /// Send the value. If the queue is full or being flushed, return false.
    pub fn send_no_wait(&'a self, val: T) -> bool {
        let mut iwrite;

        if self.flushing.load(Ordering::Relaxed) {
            return false;
        }

        loop {
            iwrite = self.iwrite.load(Ordering::SeqCst);
            let iread = self.iread.load(Ordering::SeqCst);
            let iwrite_new = (iwrite + 1) % self.data.len();

            if iwrite_new == iread {
                // Full
                return false;
            }

            if self.iwrite.compare_and_swap(iwrite, iwrite_new, Ordering::SeqCst) == iwrite {
                break;
            }
        }

        self.data[iwrite].set(Some(val));

        while self.imaxread.compare_and_swap(iwrite, (iwrite + 1) % self.data.len(),
                Ordering::SeqCst) != iwrite {
            freertos::yield_task();
        }

        return true;
    }

    /// Send the value. If the queue is full, wait until it isn't.
    pub fn send_wait(&'a self, val: T) {
        loop {
            if self.send_no_wait(val) {
                return;
            }
            freertos::yield_task();
        }
    }

    /// Receive a value. If the queue is empty, return None.
    pub fn receive_no_wait(&self) -> Option<T> {
        loop {
            let iread = self.iread.load(Ordering::SeqCst);
            let imaxread = self.imaxread.load(Ordering::SeqCst);

            if iread == imaxread {
                // Empty
                return None;
            }

            let val = self.data[iread].get().unwrap();

            if self.iread.compare_and_swap(iread, (iread + 1) % self.data.len(),
                Ordering::SeqCst) == iread {
                return Some(val);
            }
        }
    }

    /// Receive a value. If the queue is empty, wait until it isn't.
    pub fn receive_wait(&self) -> T {
        loop {
            match self.receive_no_wait() {
                Some(val) => { return val; },
                None => (),
            };
            freertos::yield_task();
        }
    }

    /// Flush the queue.
    pub fn flush(&self) {
        if self.flushing.swap(true, Ordering::Relaxed) {
            // Already being flushed by another thread, simply wait until done.
            while self.flushing.load(Ordering::Relaxed) {}
        } else {
            // Now we're flushing it
            loop {
                let iread = self.iread.load(Ordering::Relaxed);
                let imaxread = self.imaxread.load(Ordering::Relaxed);
                if iread == imaxread {
                    self.flushing.store(false, Ordering::Relaxed);
                    return;
                }
            }
        }
    }
}
