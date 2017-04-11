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

extern "C" {
    fn pvPortMalloc(sz: usize) -> *mut u8;
    fn vPortFree(pv: *mut u8);
}

#[no_mangle]
pub extern fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    return unsafe { pvPortMalloc(size) };
}

#[no_mangle]
pub extern fn __rust_deallocate(_ptr: *mut u8, _old_size: usize, _align: usize) {
    unsafe { vPortFree(_ptr) };
}

#[no_mangle]
pub extern fn __rust_reallocate(_ptr: *mut u8, _old_size: usize, _size: usize,
                                _align: usize) -> *mut u8 {
    panic!("cannot reallocate");
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, _old_size: usize,
                                        _size: usize, _align: usize) -> usize {
    panic!("cannot reallocate");
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
