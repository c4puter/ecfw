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

#![allow(mutable_transmutes)]

/*!
Static mutex. Lighter than FreeRTOS's static mutex and requires no
initialization.

This casts a const ref to self to a mut ref to self, bypassing Rust's
protections against modifying mutable statics without unsafe{} blocks.
The reason for this is that Rust's protections aren't needed - they serve
to restrict concurrent access to shared data, but everything here is by
definition and intent threadsafe.
*/

use core::mem;

use rustsys::rust_support;
use rustsys::freertos;

pub struct StaticMutex {
    pub locked: bool
}

pub struct StaticMutexLock {
    mutex: &'static StaticMutex
}

impl StaticMutex {
    pub fn take(&self) {
        unsafe {
            let mutself: &mut StaticMutex = mem::transmute(self);
            loop {
                rust_support::disable_irq();
                let was_locked = mutself.locked;
                mutself.locked = true;
                rust_support::dsb();
                rust_support::enable_irq();

                if was_locked {
                    freertos::yield_task();
                } else {
                    return;
                }
            }
        }
    }

    pub fn give(&self) {
        unsafe {
            let mutself: &mut StaticMutex = mem::transmute(self);
            rust_support::disable_irq();
            mutself.locked = false;
            rust_support::dsb();
            rust_support::enable_irq();
        }
    }

    pub fn lock(&'static self) -> StaticMutexLock {
        self.take();
        StaticMutexLock{mutex: self}
    }
}

impl Drop for StaticMutexLock {
    fn drop(&mut self) {
        self.mutex.give();
    }
}
