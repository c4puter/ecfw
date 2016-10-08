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
#![feature(lang_items)]

#[macro_use]
extern crate ec_io;
extern crate rust_support;
use core::fmt;
use rust_support::*;

#[lang="panic_fmt"]
pub extern fn panic_impl(fmt: fmt::Arguments, file: &'static str, line: u32) -> ! {
    unsafe{disable_irq();}
    println_async!("\n\n===================================");
    println_async!("PANIC");
    println_async!("file:line = {}:{}", file, line);
    print_async!  ("message   = ");
    ec_io::println_async(fmt);
    loop { }
}

#[no_mangle]
pub extern "C" fn hard_fault_printer(regs: [u32; 8]) {
    unsafe{disable_irq();}
    println_async!("\n\n===================================");
    println_async!("HARD FAULT");
    println_async!("r0  = {:08x}  r1  = {:08x}  r2  = {:08x}  r3  = {:08x}",
             regs[0], regs[1], regs[2], regs[3]);
    println_async!("r12 = {:08x}  lr  = {:08x}  pc  = {:08x}  psr = {:08x}",
             regs[4], regs[5], regs[6], regs[7]);

    println_async!("");
    unsafe {
        println_async!("BFAR = {:08x}  CFSR = {:08x}  HFSR = {:08x}",
                 readmem(0xe000ed38),
                 readmem(0xe000ed28),
                 readmem(0xe000ed2c));
        println_async!("DFSR = {:08x}  AFSR = {:08x}  SCB_SHCSR = {:08x}",
                 readmem(0xe000ed30),
                 readmem(0xe000ed3c),
                 readmem(0xe000e000 + 0x0d00 + 0x024));
                  //     SCS_BASE    SCB_off   SHCSR_off
    }
    loop{}
}
