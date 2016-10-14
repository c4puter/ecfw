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

use core::mem;
use core::fmt;

#[lang="eh_personality"] extern fn eh_personality() {}

pub trait Error: fmt::Display {
    fn description(&self) -> &str;
    fn cause(&self) -> Option<&Error> {None}
}

#[inline(always)]
pub fn nop()
{
    unsafe{asm!("nop" : : : : "volatile");}
}

#[inline(always)]
pub unsafe fn enable_irq()
{
    asm!("cpsie i" : : : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn disable_irq()
{
    asm!("cpsid i" : : : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn pendsv()
{
    writemem(0xe000ed04, (1 as u32) << (28 as u32));
}

#[inline(always)]
pub fn dsb()
{
    unsafe{asm!("dsb" :::: "volatile");}
}

#[inline(always)]
pub fn isb()
{
    unsafe{asm!("isb" :::: "volatile");}
}

#[inline(always)]
pub unsafe fn readmem(addr: u32) -> u32
{
    let p: *const u32 = mem::transmute(addr);
    return *p;
}

#[inline(always)]
pub unsafe fn writemem(addr: u32, value: u32)
{
    let p: *mut u32 = mem::transmute(addr);
    *p = value;
}

