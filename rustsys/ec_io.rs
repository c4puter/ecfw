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

use core::fmt;

extern crate bindgen_usart;
extern crate freertos;

struct UartWriter {}
struct UartWriterAsync {}

static mut stdout_queue: Option<freertos::Queue<u8>> = None;
static mut stdout_mutex: Option<freertos::Mutex> = None;

fn putc_task(q: freertos::Queue<u8>)
{
    loop {
        match q.receive(100) {
            Some(c) => unsafe {
                bindgen_usart::ec_usart_putc(c)
            },
            None => {}
        }
    }
}

pub fn putc(c: u8) {
    match unsafe{stdout_queue} {
        Some(q) => q.send(&c, 1000).unwrap(),
        None => ()
    }
}

pub fn putc_async(c: u8) {
    unsafe{bindgen_usart::ec_usart_putc(c)};
}

pub fn getc_async() -> u8 {
    unsafe{bindgen_usart::ec_usart_getc()}
}

impl<'a> fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            if *c == b'\n' {
                putc(b'\r');
            }
            putc(*c);
        }
        return Ok(());
    }
}

impl<'a> fmt::Write for UartWriterAsync {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            if *c == b'\n' {
                putc_async(b'\r');
            }
            putc_async(*c);
        }
        return Ok(());
    }
}

fn mutex_take(waitticks: usize) -> Result<(), &'static str>
{
    match unsafe{stdout_mutex} {
        Some(m) => m.take(waitticks),
        None => Err("no mutex configured yet")
    }
}

fn mutex_give() -> Result<(), &'static str>
{
    match unsafe{stdout_mutex} {
        Some(m) => m.give(),
        None => Err("no mutex configured yet")
    }
}

/// Initialize EC IO. Starts a FreeRTOS task.
pub fn init()
{
    let q = freertos::Queue::<u8>::new(72);
    let m = freertos::Mutex::new();
    unsafe {
        stdout_queue = Some(q);
        stdout_mutex = Some(m);
        bindgen_usart::ec_usart_init();
    }
    freertos::Task::new(move || { putc_task(q); }, "ec_io", 200, 0);
}

fn _print(args: fmt::Arguments) {
    fmt::write(&mut UartWriter{}, args).unwrap();
}

pub fn print(args: fmt::Arguments) {
    mutex_take(1000).unwrap();
    _print(args);
    mutex_give().unwrap();
}

pub fn println(args: fmt::Arguments) {
    mutex_take(1000).unwrap();
    _print(args);
    fmt::Write::write_str(&mut UartWriter{}, "\n").unwrap();
    mutex_give().unwrap()
}

pub fn print_async(args: fmt::Arguments) {
    fmt::write(&mut UartWriterAsync{}, args).unwrap();
}

pub fn println_async(args: fmt::Arguments) {
    print_async(args);
    fmt::Write::write_str(&mut UartWriterAsync{}, "\n").unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (ec_io::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => (ec_io::println(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_async {
    ($($arg:tt)*) => (ec_io::print_async(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_async {
    ($($arg:tt)*) => (ec_io::println_async(format_args!($($arg)*)));
}
