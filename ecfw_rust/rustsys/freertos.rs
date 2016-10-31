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
use core::sync::atomic::*;
use alloc::boxed::Box;

static TICK_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
static SUSPEND_LEVEL: AtomicUsize = ATOMIC_USIZE_INIT;

pub enum Void {}
pub type TaskHandle = u32;

#[allow(dead_code)] const pdTRUE: i32 = 1;
#[allow(dead_code)] const pdFALSE: i32 = 0;

pub struct Task { }

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

    // Utilities
    fn xPortGetFreeHeapSize() -> usize;
    fn vTaskDelay(xTicksToDelay: u32);
    fn vTaskSuspendAll();
    fn vTaskResumeAll();
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

/// Yield if tasks are not suspended, otherwise do nothing.
pub fn yield_safe() {
    unsafe{ rust_support::disable_irq(); }
    if SUSPEND_LEVEL.load(Ordering::Relaxed) == 0 {
        unsafe{ rust_support::pendsv(); }
    }
    unsafe{ rust_support::enable_irq(); }
    rust_support::dsb();
    rust_support::isb();
}

pub fn delay(nticks: u32) {
    unsafe{ vTaskDelay(nticks); }
}

/// Delay, even if the scheduler is suspended
pub fn susp_safe_delay(nticks: u32) {
    let end_tick = ticks().wrapping_add(nticks);
    // If the addition wrapped, wait for the tick counter to catch up
    while end_tick < ticks() {
        yield_safe();
    }
    while ticks() < end_tick {
        yield_safe();
    }
}

/// Get the total number of ticks elapsed since run(). This is an independent
/// tick counter that runs even when the scheduler is suspended.
pub fn ticks() -> u32 {
    TICK_COUNT.load(Ordering::Relaxed) as u32
}

pub unsafe fn suspend_all() {
    SUSPEND_LEVEL.fetch_add(1, Ordering::Relaxed);
    vTaskSuspendAll();
}

pub unsafe fn resume_all() {
    if SUSPEND_LEVEL.fetch_sub(1, Ordering::Relaxed) == 0 {
        SUSPEND_LEVEL.fetch_add(1, Ordering::Relaxed);
    } else {
        vTaskResumeAll();
    }
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

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn vApplicationTickHook()
{
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
}
