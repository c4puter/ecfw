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

extern crate bindgen_mcu;
extern crate bindgen_usart;

extern crate rust_support;
#[macro_use]
extern crate ec_io;
extern crate freertos;

pub fn delay(t: u32)
{
    for _ in 0..t {
        rust_support::nop();
    }
}

pub fn count_task(q: freertos::Queue<i32>) {
    let mut i = 0;
    loop {
        println!("handle: {:08x}", q.handle as u32);
        match q.send(&i, 100) {
            Ok(()) => { i += 1; }
            _ => {}
        }
        delay(40000);
    }
}

pub fn speak_task(q: freertos::Queue<i32>) {
    loop {
        match q.receive(100) {
            Some(i) => println!("{} number {}", "Hello, world!", i),
            None => {} }
    }
}

pub fn test(qh: u32) {
    let q = freertos::Queue::<i32>::from_handle(qh);
    println!("Handle in test: {:08x}", q.handle);
    q.send(&42, 100);
    loop{}
}

pub fn test2(qh: u32) {
    delay(50000);
    let q = freertos::Queue::<i32>::from_handle(qh);
    println!("Handle in test2: {:08x}", q.handle);
    match q.receive(100) {
        Some(i) => println!("Received {}", i),
        None => {} }
    panic!("panicpanicpanic");
    loop {}
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern "C" fn main() -> i32 {
    unsafe {
        bindgen_mcu::mcu_init();
        bindgen_mcu::board_init();
        bindgen_usart::ec_usart_init();
    }

    let q = freertos::Queue::<i32>::new(16);
    let clos1 = move || { test(q.handle); };
    let clos2 = move || { test2(q.handle); };
    let task1 = freertos::Task::new(clos1, "test1", 6000, 0);
    let task2 = freertos::Task::new(clos2, "test2", 6000, 0);
    freertos::run();
    loop {}

    return 0;
}
