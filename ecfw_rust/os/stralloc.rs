/*
 * The MIT License (MIT)
 * Copyright (c) 2017 Chris Pavlina
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

use alloc::boxed::Box;
use core::marker::PhantomData;
use core::mem;
use core::str;

/// Size of allocated blocks
const ALLOC_SZ: usize = 1024;

/// Allocator for short strings and other data with short lifetime, designed to
/// minimize heap fragmentation. Strings are allocated out of larger fixed-size
/// blocks, which are then freed at the end.
///
/// Byte arrays are freed when this StrAlloc goes out of scope.
pub struct StrAlloc<'a> {
    firstblock: Option<Box<StrAllocBlock<'a>>>,
    _pd: PhantomData<&'a StrAllocBlock<'a>>,
}

#[repr(packed)]
struct StrAllocHeader<'a> {
    _next: Option<Box<StrAllocBlock<'a>>>,
    next_index: usize,
    _pd: PhantomData<&'a StrAllocBlock<'a>>,
}

/// Size of a StrAllocHeader. This MUST be accurate, and is required because
/// mem::size_of is not a const fn. Will be checked at runtime by assertion.
const HEADER_SZ: usize = 8;

const ARRAY_SZ: usize = ALLOC_SZ - HEADER_SZ;

#[repr(packed)]
struct StrAllocBlock<'a> {
    header: StrAllocHeader<'a>,
    array: [u8; ARRAY_SZ],
}

impl<'a> StrAlloc<'a> {

    /// Return a new StrAlloc. Allocates 1 * ALLOC_SZ.
    pub fn new() -> StrAlloc<'a>
    {
        StrAlloc {
            firstblock: Some(Box::new(
                StrAllocBlock {
                    header: StrAllocHeader {
                        _next: None,
                        next_index: 0,
                        _pd: PhantomData },
                    array: [0u8; ARRAY_SZ] } )),
            _pd: PhantomData }
    }

    /// Allocate a block. If sz is too big, returns None.
    pub fn alloc(& mut self, sz: usize) -> Option<& mut [u8]>
    {
        if sz > ARRAY_SZ {
            return None;
        }

        let remaining_in_block = ARRAY_SZ -
            self.firstblock.as_ref().unwrap().header.next_index;

        if remaining_in_block <= sz {
            let fb = self.firstblock.as_mut().unwrap();

            // Allocate from this block
            let alloc_idx = fb.header.next_index;
            fb.header.next_index += sz;

            Some(&mut fb.array[alloc_idx..alloc_idx+sz])
        } else {
            // New block
            let mut prev_first: Option<Box<StrAllocBlock<'a>>> = None;
            mem::swap(&mut prev_first, &mut self.firstblock);

            self.firstblock = Some(Box::new(
                StrAllocBlock {
                    header: StrAllocHeader {
                        _next: prev_first,
                        next_index: sz,
                        _pd: PhantomData },
                    array: [0u8; ARRAY_SZ] } ));

            Some(&mut self.firstblock.as_mut().unwrap().array[0..sz])
        }
    }

    /// Shortcut function to NUL-terminate a string.
    pub fn nulterm(& mut self, s: &str) -> Option<& str>
    {
        let sb = s.as_bytes();
        let bufp = self.alloc(sb.len() + 1);

        if let Some(buf) = bufp {
            for i in 0..sb.len() {
                buf[i] = sb[i];
            }
            buf[sb.len()] = 0;
            Some(unsafe{str::from_utf8_unchecked(buf)})
        } else {
            None
        }
    }
}
