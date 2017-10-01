/*
 * c4puter embedded controller firmware
 * Copyright (C) 2017 Chris Pavlina
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

/*!
Lock-free threadsafe queue.
*/

use os;
use core::sync::atomic::*;
use core::cell::*;
use core::mem;
use core::slice;
use alloc::boxed::*;
use alloc::allocator;
use alloc::allocator::Alloc;
use alloc::heap;

pub struct Queue<'a, T> where T: 'a + Send + Copy + Sized {
    data: &'a [Cell<Option<T>>],
    _owned_data: Option<Box<[Cell<Option<T>>]>>,
    iread: AtomicUsize,
    imaxread: AtomicUsize,
    iwrite: AtomicUsize,
    flushing: AtomicBool,
    receiver: AtomicUsize,
}

macro_rules! queue_static_init {
    ( $n:expr ) => (
        repeat_expr!{$n { ::core::cell::Cell::new(None) } }
    )
}

macro_rules! queue_static_new {
    ( $( $name:ident: [$ty:ty; $n:expr] );* ) => ( $(
        static $name: $crate::os::Queue<'static, $ty> = unsafe {
            $crate::os::Queue::new_static(
                {static mut BACKING: [::core::cell::Cell<Option<$ty>>; $n]
                        = queue_static_init!($n);
                 &BACKING})
        };
    )* );

    ( $( $name:ident: [$ty:ty; $n:expr] );+; ) => (
        queue_static_new!( $( $name: [$ty; $n] );+ ); )
}

unsafe impl<'a, T> Sync for Queue<'a, T> where T: 'a + Send + Copy + Sized {}

impl<'a, T> Queue<'a, T> where T: 'a + Send + Copy + Sized {
    /// Create a new queue using a static backing array. This is unsafe because
    /// it "takes" the array; nothing else should ever touch it.
    ///
    /// Don't use this function. See queue_static_new!.
    pub const unsafe fn new_static(
        buffer: &'static [Cell<Option<T>>]) -> Queue<'a, T>
    {
        Queue {
            data: buffer,
            _owned_data: None,
            iread: ATOMIC_USIZE_INIT,
            imaxread: ATOMIC_USIZE_INIT,
            iwrite: ATOMIC_USIZE_INIT,
            flushing: ATOMIC_BOOL_INIT,
            receiver: ATOMIC_USIZE_INIT,
        }
    }

    pub fn new_dynamic(sz: usize) -> Queue<'a, T> {
        let layout = allocator::Layout::from_size_align(
            sz * mem::size_of::<Cell<Option<T>>>(), 4).unwrap();
        let buffer = unsafe{heap::Heap.alloc_zeroed(layout)}.unwrap();
        let as_slice = unsafe{
            slice::from_raw_parts_mut(buffer as *mut Cell<Option<T>>, sz)};
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
            receiver: ATOMIC_USIZE_INIT,
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

            if self.iwrite.compare_and_swap(
                iwrite, iwrite_new, Ordering::SeqCst) == iwrite
            {
                break;
            }
        }

        self.data[iwrite].set(Some(val));

        while iwrite != self.imaxread.compare_and_swap(
            iwrite, (iwrite + 1) % self.data.len(), Ordering::SeqCst)
        {
            os::yield_task();
        }

        let receiver = self.receiver.load(Ordering::SeqCst);
        if receiver != 0 {
            os::freertos::notify_give(receiver as os::freertos::TaskHandle);
        }

        return true;
    }

    /// Send the value. If the queue is full, wait until it isn't.
    pub fn send_wait(&'a self, val: T) {
        loop {
            if self.send_no_wait(val) {
                return;
            }
            os::yield_task();
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
            os::yield_task();
        }
    }

    /// Register the current task as the queue's singular notified receiver.
    /// Panics if there is already a receiver.
    ///
    /// It is still possible to have multiple queue consumers, but only one can
    /// receive notifications via FreeRTOS.
    pub fn register_receiver(&self) {
        let task = os::freertos::this_task();
        let previous_task = self.receiver.swap(task as usize, Ordering::SeqCst);

        assert!(previous_task == 0);
    }

    /// Unregister the current task as the queue's notified receiver.
    /// Panics if the current task is not the one registered.
    pub fn unregister_receiver(&self) {
        let previous_task = self.receiver.swap(0usize, Ordering::SeqCst);
        assert!(previous_task == os::freertos::this_task() as usize);
    }

    /// Receive a value. If the queue is empty, block until it isn't. BEWARE, if
    /// the current task has not registered as receiver, this will deadlock.
    pub fn receive_wait_blocking(&self) -> T {
        loop {
            match self.receive_no_wait() {
                Some(val) => { return val; },
                None => {},
            };
            os::freertos::notify_take(os::freertos::CounterAction::Decrement, 1000);
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
