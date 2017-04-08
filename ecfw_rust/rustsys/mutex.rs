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

use rustsys::rwlock::*;

pub struct Mutex<T: Sized + Sync> {
    rwlock: RwLock<T>,
}

pub type MutexLock<'a, T> = RwLockWriter<'a, T>;

impl<T> Mutex<T> where T: Sized + Sync {
    pub const fn new(data: T) -> Mutex<T> {
        Mutex {rwlock: RwLock::new(data)}
    }

    pub fn lock(&self) -> MutexLock<T> {
        self.rwlock.write()
    }

    pub fn try_lock(&self) -> Option<MutexLock<T>> {
        self.rwlock.try_write()
    }

    pub fn lock_timeout(&self, nticks: u32) -> Option<MutexLock<T>> {
        self.rwlock.write_timeout(nticks)
    }

}
