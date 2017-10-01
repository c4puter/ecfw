/*
 * c4puter embedded controller firmware
 * Copyright (C) 2017 Chris Pavlina
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
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

