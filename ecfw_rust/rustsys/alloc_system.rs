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

extern "C" {
    fn pvPortMalloc(sz: usize) -> *mut u8;
    fn vPortFree(pv: *mut u8);
    fn memset(p: *mut u8, val: i32, size: usize) -> *mut u8;
}

#[no_mangle]
pub extern fn __rust_alloc(size: usize, _align: usize, _err: *mut u8) -> *mut u8 {
    let p = unsafe { pvPortMalloc(size) };
    debug!(DEBUG_ALLOC, "allocate {} bytes at 0x{:08x}", size, (p as usize));
    p
}

#[no_mangle]
pub extern fn __rust_alloc_zeroed(size: usize, align: usize, err: *mut u8) -> *mut u8 {
    let p = __rust_alloc(size, align, err);
    unsafe { memset(p, 0, size) }
}

#[no_mangle]
pub extern fn __rust_dealloc(ptr: *mut u8, _old_size: usize, _align: usize) {
    debug!(DEBUG_ALLOC, "free 0x{:08x}", (ptr as usize));
    unsafe { vPortFree(ptr) };
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}

#[no_mangle]
pub extern fn __rust_oom(_err: *const u8) -> ! {
    panic!("out of memory");
}
