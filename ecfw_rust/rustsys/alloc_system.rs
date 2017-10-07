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
    fn memcpy(dest: *mut u8, src: *const u8, size: usize) -> *mut u8;
}

#[no_mangle]
pub unsafe extern fn __rust_alloc(size: usize, align: usize, _err: *mut u8) -> *mut u8
{
    if align > 8 {
        panic!("alloc requested alignment greater than 8 ({})", align);
    } else if align.count_ones() != 1 {
        panic!("alloc requested alignment non-power-of-2 ({})", align);
    }

    // Size must be a multiple of 8 bytes or the FreeRTOS allocator can come
    // unaligned. Gah.
    let size_aligned = size + 7 & !7;

    let p = pvPortMalloc(size_aligned);
    debug!(DEBUG_ALLOC, "allocate {:4} bytes at 0x{:08x} (align {}, actual 8)",
        size, (p as usize), align);
    p
}

#[no_mangle]
pub unsafe extern fn __rust_alloc_zeroed(size: usize, align: usize, err: *mut u8) -> *mut u8 {
    let p = __rust_alloc(size, align, err);
    memset(p, 0, size)
}

#[no_mangle]
pub unsafe extern fn __rust_dealloc(ptr: *mut u8, _old_size: usize, _align: usize) {
    debug!(DEBUG_ALLOC, "free 0x{:08x}", (ptr as usize));
    vPortFree(ptr);
}

#[no_mangle]
pub unsafe fn __rust_realloc(
        ptr: *mut u8,
        old_size: usize, old_align: usize,
        new_size: usize, new_align: usize, err: *mut u8) -> *mut u8
{
    if old_align != new_align {
        panic!("realloc requested change in alignment ({} to {})",
               old_align, new_align);
    }

    if new_size < old_size {
        debug!(DEBUG_ALLOC, "realloc 0x{:08x} to lower size - doing nothing",
               (ptr as usize));
        return ptr;
    }

    debug!(DEBUG_ALLOC, "realloc 0x{:08x} from {} to {}",
           (ptr as usize), old_size, new_size);

    let new_ptr = __rust_alloc(new_size, new_align, err);

    memcpy(new_ptr, ptr, old_size);

    __rust_dealloc(ptr, old_size, old_align);

    new_ptr
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}

#[no_mangle]
pub extern fn __rust_oom(_err: *const u8) -> ! {
    panic!("out of memory");
}
