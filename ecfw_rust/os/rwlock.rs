/*
 * The MIT License (MIT)
 * Copyright (c) 2017 Chris Pavlina
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

use os;
use core::sync::atomic::*;
use core::cell::UnsafeCell;
use core::ops::*;

pub struct RwLock<T: Sized + Sync> {
    data: UnsafeCell<T>,
    readers: AtomicUsize,
    writers: AtomicUsize,
}

impl<T> RwLock<T> where T: Sized + Sync {
    pub const fn new(data: T) -> RwLock<T> {
        RwLock {
            data: UnsafeCell::new(data),
            readers: ATOMIC_USIZE_INIT,
            writers: ATOMIC_USIZE_INIT,
        }
    }

    pub fn try_write(&self) -> Option<RwLockWriter<T>> {
        let prev_writers = self.writers.fetch_add(1, Ordering::SeqCst);

        if prev_writers > 0 {
            self.writers.fetch_sub(1, Ordering::SeqCst);
            return None;
        }

        let readers = self.readers.load(Ordering::SeqCst);

        if readers > 0 {
            self.writers.fetch_sub(1, Ordering::SeqCst);
            return None;
        }

        Some(RwLockWriter::<T>{
            data: unsafe{ self.data.get().as_mut().unwrap() },
            lock: self })
    }

    pub fn try_read(&self) -> Option<RwLockReader<T>> {
        self.readers.fetch_add(1, Ordering::SeqCst);
        let writers = self.writers.load(Ordering::SeqCst);

        if writers > 0 {
            self.readers.fetch_sub(1, Ordering::SeqCst);
            return None;
        }

        Some(RwLockReader::<T>{
            data: unsafe{ self.data.get().as_ref().unwrap() },
            lock: self })
    }

    pub fn write(&self) -> RwLockWriter<T> {
        loop {
            if let Some(wr) = self.try_write() {
                return wr;
            } else {
                os::yield_task();
            }
        }
    }

    pub fn read(&self) -> RwLockReader<T> {
        loop {
            if let Some(rd) = self.try_read() {
                return rd;
            } else {
                os::yield_task();
            }
        }
    }

    pub fn write_timeout(&self, nticks: u32) -> Option<RwLockWriter<T>> {
        let end_tick = os::ticks().wrapping_add(nticks);
        while end_tick < os::ticks() {
            if let Some(wr) = self.try_write() {
                return Some(wr);
            }
            os::yield_task();
        }
        while os::ticks() < end_tick {
            if let Some(wr) = self.try_write() {
                return Some(wr);
            }
            os::yield_task();
        }
        None
    }

    pub fn read_timeout(&self, nticks: u32) -> Option<RwLockReader<T>> {
        let end_tick = os::ticks().wrapping_add(nticks);
        while end_tick < os::ticks() {
            if let Some(rd) = self.try_read() {
                return Some(rd);
            }
            os::yield_task();
        }
        while os::ticks() < end_tick {
            if let Some(rd) = self.try_read() {
                return Some(rd);
            }
            os::yield_task();
        }
        None
    }

    pub fn drop_writer(&self) {
        let prev_writers = self.writers.fetch_sub(1, Ordering::SeqCst);
        assert!(prev_writers > 0);
    }

    pub fn drop_reader(&self) {
        let prev_readers = self.readers.fetch_sub(1, Ordering::SeqCst);
        assert!(prev_readers > 0);
    }
}

unsafe impl<T> Sync for RwLock<T> where T: Sync {}

pub struct RwLockWriter<'a, T: Sized + Sync + 'a> {
    pub data: &'a mut T,
    pub lock: &'a RwLock<T>,
}

impl<'a, T> Deref for RwLockWriter<'a, T> where T: Sized + Sync + 'a {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> DerefMut for RwLockWriter<'a, T> where T: Sized + Sync + 'a {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T> Drop for RwLockWriter<'a, T> where T: Sized + Sync + 'a {
    fn drop(&mut self) {
        self.lock.drop_writer();
    }
}

pub struct RwLockReader<'a, T: Sized + Sync + 'a> {
    pub data: &'a T,
    pub lock: &'a RwLock<T>,
}

impl<'a, T> Deref for RwLockReader<'a, T> where T: Sized + Sync + 'a {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> Drop for RwLockReader<'a, T> where T: Sized + Sync + 'a {
    fn drop(&mut self) {
        self.lock.drop_reader();
    }
}
