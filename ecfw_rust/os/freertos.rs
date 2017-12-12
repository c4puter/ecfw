// c4puter embedded controller firmware
// Copyright (C) 2017 Chris Pavlina
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//

//! FreeRTOS wrappers and related functions

#![allow(improper_ctypes, non_upper_case_globals)]

use rustsys::rust_support;
use core::ptr;
use core::str;
use core::slice;
use core::mem;
use core::cell::UnsafeCell;
use alloc::boxed::Box;
use ctypes::c_void;

// Optimization because tick counts are sometimes checked in fairly tight
// places - we don't actually need an Atomic here, because it's one word
// long and only modified in one place. Use an UnsafeCell instead and
// read/write with ptr::volatile_load/volatile_store.
struct TickCount(UnsafeCell<u32>);
unsafe impl Sync for TickCount {}
static TICK_COUNT: TickCount = TickCount(UnsafeCell::new(0));

pub type TaskHandle = u32;

#[allow(dead_code)] const pdTRUE: i32 = 1;
#[allow(dead_code)] const pdFALSE: i32 = 0;
#[allow(dead_code)] const pdPASS: i32 = 1;
#[allow(dead_code)] const pdFAIL: i32 = 0;

pub struct Task {}

#[allow(unused)]
#[repr(C)]
enum NotifyAction {
    NoAction = 0,
    SetBits,
    Increment,
    SetValueWithOverwrite,
    SetValueWithoutOverwrite,
}

extern "C" {
    fn xTaskCreate(
        pxTaskCode: extern "C" fn(task: *mut c_void),
        pcName: *const u8,
        ulStackDepth: u32,
        pvParameters: *mut c_void,
        uxPriority: u32,
        puxStackBuffer: *const c_void,
        pxTaskBuffer: *const c_void,
    );
    fn vTaskStartScheduler();
    fn strlen(s: *const u8) -> usize;

    // Utilities
    fn xPortGetFreeHeapSize() -> usize;
    fn xPortGetMinimumEverFreeHeapSize() -> usize;
    fn vTaskDelay(xTicksToDelay: u32);
    fn vTaskDelayUntil(pxPreviousWakeTime: *mut u32, xTimeIncrement: u32);
    fn xTaskGetTickCount() -> u32;
    fn vTaskSuspendAll();
    fn xTaskResumeAll() -> usize;
    fn xTaskGetCurrentTaskHandle() -> TaskHandle;

    // Notification functions
    fn xTaskGenericNotify(
        xTaskToNotify: TaskHandle,
        ulValue: u32,
        eAction: NotifyAction,
        pulPreviousNotificationValue: *mut u32,
    ) -> i32;

    #[allow(unused)]
    fn xTaskGenericNotifyFromISR(
        xTaskToNotify: TaskHandle,
        ulValue: u32,
        eAction: NotifyAction,
        pulPreviousNotificationValue: *mut u32,
        pxHigherPriorityTaskWoken: *mut i32,
    ) -> i32;

    // xTaskNotifyGive(task) = xTaskGenericNotify(task, 0,
    //  NotifyAction::Increment, ptr::null_mut())

    // Return pdPASS if a notification was received, otherwise pdFAIL
    #[allow(unused)]
    fn xTaskNotifyWait(
        ulBitsToClearOnEntry: u32,
        ulBitsToClearOnExit: u32,
        pulNotificationValue: *mut u32,
        xTicksToWait: u32,
    ) -> i32;

    // Wait for the notification count to be nonzero, then either clear or
    // decrement it.
    //
    // @param xClearCountOnExit: pdTRUE to clear, pdFALSE to decrement
    // @param xTicksToWait: timeout
    // @return value of counter before decremented or cleared
    fn ulTaskNotifyTake(xClearCountOnExit: i32, xTicksToWait: u32) -> u32;
}

extern "C" fn task_wrapper<F>(task: *mut c_void)
where
    F: Fn(),
{
    let tboxptr = task as *mut Box<Fn()>;
    let pclos: Box<Box<F>> = unsafe { mem::transmute(tboxptr) };
    pclos();
}

impl Task {
    /// Create and start a task
    ///
    /// # Arguments
    /// - `f` - closure or function to run as the task
    /// - `name` - task name
    /// - `stackdepth` - size of the task's stack in words
    /// - `priority` - task priority
    pub fn new<F>(f: F, name: &str, stackdepth: usize, priority: u32) -> Task
    where
        F: Fn(),
    {
        let fbox = Box::new(Box::new(f));
        unsafe {
            xTaskCreate(
                task_wrapper::<F>,
                name.as_bytes().as_ptr(),
                stackdepth as u32,
                Box::into_raw(fbox) as *mut c_void,
                priority,
                ptr::null(),
                ptr::null(),
            );
        }
        Task {}
    }
}

/// Launch the task scheduler.
pub fn run()
{
    unsafe {
        vTaskStartScheduler();
    }
}

/// Return the number of bytes of free heap space.
pub fn get_free_heap() -> usize
{
    unsafe { xPortGetFreeHeapSize() }
}

/// Return the smallest number of bytes of free heap space ever.
pub fn get_worst_free_heap() -> usize
{
    unsafe { xPortGetMinimumEverFreeHeapSize() }
}

/// Yield the current task to the next.
pub fn yield_task()
{
    unsafe {
        rust_support::pendsv();
    }
    rust_support::dsb();
    rust_support::isb();
}

/// Delay the current task for the given number of millisecond ticks.
pub fn delay(nticks: u32)
{
    unsafe {
        vTaskDelay(nticks);
    }
}

/// Delay just enough to make the task run with a fixed period.
///
/// # Arguments
/// - `lastwake` - The last tick count when the task woke. This is written with
///                each `delay_period` call, and must be initialized to
///                `ticks_running()`.
/// - `period` - Delay period in millisecond ticks.
pub fn delay_period(lastwake: &mut u32, period: u32)
{
    unsafe {
        vTaskDelayUntil(lastwake as *mut u32, period);
    }
}

/// Delay a specified number of millisecond ticks, even if the scheduler is
/// suspended.
pub fn susp_safe_delay(nticks: u32)
{
    let end_tick = ticks().wrapping_add(nticks);
    // If the addition wrapped, wait for the tick counter to catch up
    while end_tick < ticks() {
        yield_task();
    }
    while ticks() < end_tick {
        yield_task();
    }
}

/// Get the total number of ticks elapsed since `run()`. This is an independent
/// tick counter that runs even when the scheduler is suspended.
pub fn ticks() -> u32
{
    // Safe: value is one word long and always in a valid state.
    unsafe{ptr::read_volatile(TICK_COUNT.0.get())}
}

/// Get the total number of ticks elapsed since `run()`. This is the FreeRTOS
/// counter and does not run while the scheduler is suspended.
pub fn ticks_running() -> u32
{
    unsafe { xTaskGetTickCount() }
}

/// Suspend all tasks. Nests safely.
pub unsafe fn suspend_all()
{
    vTaskSuspendAll();
}

/// Resume all tasks. Nests safely; panics if not suspended.
pub unsafe fn resume_all()
{
    xTaskResumeAll();
}

/// Get the handle of the currently running task.
pub fn this_task() -> TaskHandle
{
    unsafe { xTaskGetCurrentTaskHandle() }
}

/// Increment the notification counter of a task from within another task.
pub fn notify_give(task: TaskHandle)
{
    unsafe {
        xTaskGenericNotify(task, 0, NotifyAction::Increment, ptr::null_mut());
    }
}

/// Incremnt hte notification counter of a task from within an ISR.
pub fn notify_give_from_isr(task: TaskHandle)
{
    unsafe {
        xTaskGenericNotifyFromISR(
            task,
            0,
            NotifyAction::Increment,
            ptr::null_mut(),
            ptr::null_mut(),
        );
    }
}

/// Counter behavior for notify_take
pub enum CounterAction {
    Clear,
    Decrement,
}

/// Wait for the notification counter of the current task to become nonzero.
///
/// # Arguments
/// - `counter_action` - what to do with the counter when it becomes nonzero
/// - `timeout_ticks` - how many ticks before timeout
///
/// # Return
/// Value of the counter after notification. If zero, timeout occurred.
pub fn notify_take(counter_action: CounterAction, timeout_ticks: u32) -> u32
{
    unsafe {
        ulTaskNotifyTake(
            match counter_action {
                CounterAction::Clear => pdTRUE,
                CounterAction::Decrement => pdFALSE,
            },
            timeout_ticks,
        )
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[doc(hidden)]
pub extern "C" fn vApplicationStackOverflowHook(
    taskhnd: *const c_void,
    pname: *const u8,
)
{
    let _ = taskhnd;
    // Safe: originally came from &str
    let name = unsafe {
        str::from_utf8_unchecked(slice::from_raw_parts(pname, strlen(pname)))
    };
    panic!("Stack overflow in task: {}", name);
}

#[no_mangle]
#[allow(non_snake_case)]
#[doc(hidden)]
pub extern "C" fn vApplicationMallocFailedHook()
{
    panic!("Out of memory");
}

#[no_mangle]
#[allow(non_snake_case)]
#[doc(hidden)]
pub extern "C" fn vApplicationTickHook()
{
    // Safe: value is one word long and only modified here, so this will
    // always be valid.
    unsafe {
        let tc = ptr::read_volatile(TICK_COUNT.0.get()).wrapping_add(1);
        ptr::write_volatile(TICK_COUNT.0.get(), tc);
    }
}
