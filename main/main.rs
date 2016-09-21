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

pub fn test_loop() {
    let mut i = 0;
    loop {
        println!("{} number {}", "Hello, world!", i);
        i += 1;
        delay(40000);
    }
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    unsafe {
        bindgen_mcu::mcu_init();
        bindgen_mcu::board_init();
        bindgen_usart::ec_usart_init();
    }

    let led_task = freertos::Task::new(test_loop, "test_loop", 200, 0);
    freertos::run();
    loop {}
    return 0;
}
