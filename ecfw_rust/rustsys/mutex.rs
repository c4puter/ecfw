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
Static mutex. Lighter than FreeRTOS's static mutex and requires no
initialization.
*/

use rustsys::freertos;
use core::sync::atomic::*;

pub struct Mutex {
    pub locked: AtomicBool
}

pub struct MutexLock<'a> {
    mutex: &'a Mutex
}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex {locked: ATOMIC_BOOL_INIT}
    }

    pub fn take(&self) {
        loop {
            let was_locked = self.locked.swap(true, Ordering::SeqCst);

            if was_locked {
                freertos::yield_task();
            } else {
                return;
            }
        }
    }

    pub fn give(&self) {
        self.locked.store(false, Ordering::SeqCst);
    }

    pub fn lock(&self) -> MutexLock {
        self.take();
        MutexLock{mutex: self}
    }
}

impl <'a> Drop for MutexLock<'a> {
    fn drop(&mut self) {
        self.mutex.give();
    }
}
