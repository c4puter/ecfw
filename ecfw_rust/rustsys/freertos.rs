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

#![allow(improper_ctypes, non_upper_case_globals)]

use rustsys::rust_support;
use core::ptr;
use core::str;
use core::slice;
use core::mem;
use core::marker;
use alloc::boxed::Box;

pub enum Void {}
type QueueHandle = u32;
pub type TaskHandle = u32;

const pdTRUE: i32 = 1;
const pdFALSE: i32 = 0;
const errQUEUE_FULL: i32 = 0;
const queueSEND_TO_BACK: i32 = 0;
const queueSEND_TO_FRONT: i32 = 1;
const queueQUEUE_TYPE_BASE: u8 = 0;
const queueQUEUE_TYPE_MUTEX: u8 = 1;
const semGIVE_BLOCK_TIME: usize = 0;

#[derive(Copy, Clone)]
pub struct Queue<T> {
    handle: QueueHandle,
    phantom: marker::PhantomData<T>,
}

pub struct Task { }

#[derive(Copy, Clone)]
pub struct Mutex {
    handle: QueueHandle
}

pub struct MutexLock {
    mutex: Mutex
}

extern "C" {
    fn xTaskCreate(
        pxTaskCode: extern "C" fn(task: *mut Void),
        pcName: *const u8,
        ulStackDepth: u32,
        pvParameters: *mut Void,
        uxPriority: u32,
        puxStackBuffer: *const Void,
        pxTaskBuffer: *const Void);
    fn vTaskStartScheduler();
    fn strlen(s: *const u8) -> usize;

    // Queue management
    fn xQueueGenericCreate(queuelen: usize, itemsize: usize, qtype: u8) -> QueueHandle;
    fn xQueueGenericSend(queue: QueueHandle, item: *const Void, waitticks: usize, copypos: i32) -> i32;
    fn xQueueGenericSendFromISR(
        xQueue: QueueHandle,
        pvItemToQueue: *const Void,
        pxHigherPriorityTaskWoken: *mut i32,
        xCopyPosition: i32 ) -> i32;
    fn xQueueGenericReceive(queue: QueueHandle, item: *mut Void, waitticks: usize, peek: i32) -> i32;
    fn uxQueueMessagesWaiting(queue: QueueHandle) -> usize;
    fn uxQueueSpacesAvailable(queue: QueueHandle) -> usize;
    fn xQueueReset(queue: QueueHandle) -> i32; // always returns pdPASS

    fn xQueueCreateMutex(qtype: u8) -> QueueHandle;

    // Utilities
    fn xPortGetFreeHeapSize() -> usize;
    fn vTaskDelay(xTicksToDelay: u32);
}

extern "C" fn task_wrapper<F>(task: *mut Void) where F: Fn() {
    let tboxptr = task as *mut Box<Fn()>;
    let pclos: Box<Box<F>> = unsafe{mem::transmute(tboxptr)};
    pclos();
}

impl Task {
    pub fn new<F>(f: F, name: &str, stackdepth: usize, priority: u32) -> Task
        where F: Fn()
    {
        let fbox = Box::new(Box::new(f));
        unsafe {
            xTaskCreate(task_wrapper::<F>, name.as_bytes().as_ptr(), stackdepth as u32,
                Box::into_raw(fbox) as *mut Void, priority, ptr::null(), ptr::null());
        }
        Task{}
    }
}

impl <T> Queue<T> {
    pub fn new(len: usize) -> Queue<T> {
        let itemsize = mem::size_of::<T>();
        let qhandle = unsafe{ xQueueGenericCreate(len, itemsize, queueQUEUE_TYPE_BASE) };
        return Queue::<T> { handle: qhandle, phantom: marker::PhantomData };
    }


    fn send_generic(&self, item: &T, waitticks: usize, copypos: i32) -> Result<(), &str> {
        let res = unsafe {
            xQueueGenericSend(self.handle, mem::transmute(item), waitticks, copypos)
        };
        return match res {
            pdTRUE => Ok(()),
            errQUEUE_FULL => Err("queue full"),
            _ => Err("unknown queue error")
        };
    }

    fn send_generic_from_isr(&self, item: &T, copypos: i32) -> Result<(), &str> {
        let res = unsafe {
            xQueueGenericSendFromISR(self.handle, mem::transmute(item), ptr::null_mut(), copypos)
        };
        return match res {
            pdTRUE => Ok(()),
            errQUEUE_FULL => Err("queue full"),
            _ => Err("unknown queue error")
        };
    }

    pub fn send(&self, item: &T, waitticks: usize) -> Result<(), &str> {
        return self.send_generic(item, waitticks, queueSEND_TO_BACK);
    }

    pub fn send_to_front(&self, item: &T, waitticks: usize) -> Result<(), &str> {
        return self.send_generic(item, waitticks, queueSEND_TO_FRONT);
    }

    pub fn send_from_isr(&self, item: &T) -> Result<(), &str> {
        return self.send_generic_from_isr(item, queueSEND_TO_BACK);
    }

    fn receive_generic(&self, waitticks: usize, peek: bool) -> Option<T> {
        let mut buf: T = unsafe{ mem::zeroed() };
        let res = unsafe { xQueueGenericReceive(
                self.handle, &mut buf as *mut T as *mut Void, waitticks, peek as i32) };
        return match res {
            pdTRUE => Some(buf),
            _ => None,
        };
    }

    pub fn receive(&self, waitticks: usize) -> Option<T> {
        return self.receive_generic(waitticks, false);
    }

    pub fn peek(&self, waitticks: usize) -> Option<T> {
        return self.receive_generic(waitticks, true);
    }

    pub fn waiting(&self) -> usize {
        return unsafe{ uxQueueMessagesWaiting(self.handle) };
    }

    pub fn available(&self) -> usize {
        return unsafe{ uxQueueSpacesAvailable(self.handle) };
    }

    pub fn reset(&self) {
        unsafe{ xQueueReset(self.handle); }
    }
}

impl Mutex {
    pub fn new() -> Mutex {
        let hnd = unsafe{xQueueCreateMutex(queueQUEUE_TYPE_MUTEX)};
        Mutex{handle: hnd}
    }

    pub fn take(&self, waitticks: usize) -> Result<(), &'static str> {
        let res = unsafe{ xQueueGenericReceive(
                self.handle, ptr::null_mut(), waitticks, pdFALSE ) };
        return match res {
            pdTRUE => Ok(()),
            _ => Err("timeout")
        };
    }

    pub fn give(&self) -> Result<(), &'static str> {
        let res = unsafe {
            xQueueGenericSend(self.handle, ptr::null(), semGIVE_BLOCK_TIME, queueSEND_TO_BACK)
        };
        return match res {
            pdTRUE => Ok(()),
            _ => Err("timeout")
        };
    }

    pub fn lock(&self, waitticks: usize) -> Result<MutexLock, &'static str> {
        match self.take(waitticks) {
            Ok(_) => Ok(MutexLock{mutex: *self}),
            Err(e) => Err(e),
        }
    }
}

impl Drop for MutexLock {
    fn drop(&mut self) {
        self.mutex.give().unwrap();
    }
}

pub fn run() {
    unsafe {
        vTaskStartScheduler();
    }
}

pub fn get_free_heap() -> usize {
    unsafe { xPortGetFreeHeapSize() }
}

pub fn yield_task() {
    unsafe{ rust_support::pendsv(); }
    rust_support::dsb();
    rust_support::isb();
}

pub fn delay(ticks: u32) {
    unsafe{ vTaskDelay(ticks); }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn vApplicationStackOverflowHook(taskhnd: *const Void, pname: *const u8)
{
    let _ = taskhnd;
    let name = unsafe {
        str::from_utf8_unchecked(
            slice::from_raw_parts(pname, strlen(pname))) };
    panic!("Stack overflow in task: {}", name);
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn vApplicationMallocFailedHook()
{
    panic!("Out of memory");
}
