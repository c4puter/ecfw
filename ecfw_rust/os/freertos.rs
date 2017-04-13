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

pub enum Void {}
pub type TaskHandle = u32;

#[allow(dead_code)] const pdTRUE: i32 = 1;
#[allow(dead_code)] const pdFALSE: i32 = 0;
#[allow(dead_code)] const pdPASS: i32 = 1;
#[allow(dead_code)] const pdFAIL: i32 = 0;

pub struct Task { }

#[allow(unused)]
#[repr(C)]
enum NotifyAction {
    NoAction = 0,
    SetBits,
    Increment,
    SetValueWithOverwrite,
    SetValueWithoutOverwrite
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

    // Utilities
    fn xPortGetFreeHeapSize() -> usize;
    fn xPortGetMinimumEverFreeHeapSize() -> usize;
    fn vTaskDelay(xTicksToDelay: u32);
    fn vTaskDelayUntil(
        pxPreviousWakeTime: *mut u32,
        xTimeIncrement: u32);
    fn xTaskGetTickCount() -> u32;
    fn vTaskSuspendAll();
    fn vTaskResumeAll();
    fn xTaskGetCurrentTaskHandle() -> TaskHandle;

    // Notification functions
    fn xTaskGenericNotify(
        xTaskToNotify: TaskHandle,
        ulValue: u32,
        eAction: NotifyAction,
        pulPreviousNotificationValue: *mut u32) -> i32;

    #[allow(unused)]
    fn xTaskGenericNotifyFromISR(
        xTaskToNotify: TaskHandle,
        ulValue: u32,
        eAction: NotifyAction,
        pulPreviousNotificationValue: *mut u32,
        pxHigherPriorityTaskWoken: *mut i32) -> i32;

    // xTaskNotifyGive(task) = xTaskGenericNotify(task, 0,
    //  NotifyAction::Increment, ptr::null_mut())

    // Return pdPASS if a notification was received, otherwise pdFAIL
    #[allow(unused)]
    fn xTaskNotifyWait(
        ulBitsToClearOnEntry: u32,
        ulBitsToClearOnExit: u32,
        pulNotificationValue: *mut u32,
        xTicksToWait: u32) -> i32;

    // Wait for the notification count to be nonzero, then either clear or
    // decrement it.
    //
    // @param xClearCountOnExit: pdTRUE to clear, pdFALSE to decrement
    // @param xTicksToWait: timeout
    // @return value of counter before decremented or cleared
    fn ulTaskNotifyTake(
        xClearCountOnExit: i32,
        xTicksToWait: u32) -> u32;
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

pub fn get_worst_free_heap() -> usize {
    unsafe { xPortGetMinimumEverFreeHeapSize() }
}

pub fn yield_task() {
    unsafe{ rust_support::pendsv(); }
    rust_support::dsb();
    rust_support::isb();
}

pub fn delay(nticks: u32) {
    unsafe{ vTaskDelay(nticks); }
}

/// Delay just enough to make the task run with a fixed period.
/// @param lastwake - the last tick count when the task woke. This is written,
///     and must be initialized to ticks_running().
/// @param period - delay period in ticks.
pub fn delay_period(lastwake: &mut u32, period: u32) {
    unsafe{ vTaskDelayUntil(lastwake as *mut u32, period); }
}

/// Delay, even if the scheduler is suspended
pub fn susp_safe_delay(nticks: u32) {
    let end_tick = ticks().wrapping_add(nticks);
    // If the addition wrapped, wait for the tick counter to catch up
    while end_tick < ticks() {
        yield_task();
    }
    while ticks() < end_tick {
        yield_task();
    }
}

/// Get the total number of ticks elapsed since run(). This is an independent
/// tick counter that runs even when the scheduler is suspended.
pub fn ticks() -> u32 {
    TICK_COUNT.load(Ordering::Relaxed) as u32
}

/// Get the total number of ticks elapsed since run(). This is the FreeRTOS
/// counter and does not run while the scheduler is suspended.
pub fn ticks_running() -> u32 {
    unsafe{ xTaskGetTickCount() }
}

pub unsafe fn suspend_all() {
    vTaskSuspendAll();
}

pub unsafe fn resume_all() {
    vTaskResumeAll();
}

/// Get the handle of the currently running task
pub fn this_task() -> TaskHandle {
    unsafe {
        xTaskGetCurrentTaskHandle()
    }
}

/// Increment the notification counter of a task.
pub fn notify_give(task: TaskHandle) {
    unsafe {
        xTaskGenericNotify(task, 0, NotifyAction::Increment, ptr::null_mut());
    }
}

/// Counter behavior for notify_take
pub enum CounterAction {
    Clear,
    Decrement
}

/// Wait for the notification counter of the current task to become nonzero.
/// @param counter_action - what to do with the counter when it becomes nonzero
/// @param timeout_ticks - how many ticks before timeout
/// @return value of the counter after notification. If zero, timed out
pub fn notify_take(counter_action: CounterAction, timeout_ticks: u32) -> u32 {
    unsafe {
        ulTaskNotifyTake(
            match counter_action {
                CounterAction::Clear => pdTRUE,
                CounterAction::Decrement => pdFALSE },
            timeout_ticks)
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
