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

struct UartWriter {}

impl<'a> fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            if *c == b'\n' {
                unsafe { bindgen_usart::ec_usart_putc(b'\r'); }
            }
            unsafe{ bindgen_usart::ec_usart_putc(*c); }
        }
        return Ok(());
    }
}

static mut uartwriter: UartWriter = UartWriter{};

pub fn _print(args: fmt::Arguments) {
    unsafe{ fmt::write(&mut uartwriter, args).unwrap(); }
}

pub fn _println(args: fmt::Arguments) {
    _print(args);
    unsafe{ fmt::Write::write_str(&mut uartwriter, "\n").unwrap(); }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (ec_io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => (ec_io::_println(format_args!($($arg)*)));
}
