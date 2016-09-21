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

#![no_std]
#![allow(improper_ctypes)]

#[macro_use]
extern crate ec_io;
use core::ptr;
use core::str;
use core::slice;

pub enum Void {}

#[allow(dead_code)]
pub struct Task<'a> {
    task: fn(),
    name: &'a str,
    stackdepth: usize,
    priority: i32,
}

extern "C" {
    fn xTaskGenericCreate(
        pvTaskCode: extern "C" fn(task: fn()),
        pcName: *const u8,
        usStackDepth: u16,
        pvParameters: fn(),
        uxPriority: i32,
        pvCreatedTask: *const Void,
        puxStackBuffer: *const Void,
        xRegions: *const Void);
    fn vTaskStartScheduler();
    fn strlen(s: *const u8) -> usize;
}

extern "C" fn task_wrapper(task: fn()) {
    task();
}

impl<'a> Task<'a> {
    pub fn new(task: fn(), name: &'a str, stackdepth: usize, priority: i32) -> Task<'a> {
        unsafe {
            xTaskGenericCreate(task_wrapper, name.as_bytes().as_ptr(), stackdepth as u16,
                task, priority, ptr::null(), ptr::null(), ptr::null());
        }
        return Task { task: task, name: name, stackdepth: stackdepth, priority: priority };
    }
}

pub fn run() {
    unsafe {
        vTaskStartScheduler();
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
    println!("\n\nSTACK OVERFLOW IN {}", name);
    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn vApplicationMallocFailedHook()
{
    println!("\n\nOUT OF MEMORY");
    loop{}
}
