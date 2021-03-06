// c4puter embedded controller firmware
// Copyright (C) 2017 Chris Pavlina
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//

use alloc::boxed::Box;
use core::marker::PhantomData;
use core::mem;
use core::str;
use messages::*;

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
            firstblock: Some(Box::new(StrAllocBlock {
                header: StrAllocHeader {
                    _next: None,
                    next_index: 0,
                    _pd: PhantomData,
                },
                array: [0u8; ARRAY_SZ],
            })),
            _pd: PhantomData,
        }
    }

    /// Allocate a block. If sz is too big, returns None.
    pub fn alloc(&mut self, sz: usize) -> Result<&mut [u8], Error>
    {
        if sz > ARRAY_SZ {
            return Err(ERR_STRLEN);
        }

        let remaining_in_block =
            ARRAY_SZ - self.firstblock.as_ref().unwrap().header.next_index;

        if remaining_in_block <= sz {
            let fb = self.firstblock.as_mut().unwrap();

            // Allocate from this block
            let alloc_idx = fb.header.next_index;
            fb.header.next_index += sz;

            Ok(&mut fb.array[alloc_idx .. alloc_idx + sz])
        } else {
            // New block
            let mut prev_first: Option<Box<StrAllocBlock<'a>>> = None;
            mem::swap(&mut prev_first, &mut self.firstblock);

            self.firstblock = Some(Box::new(StrAllocBlock {
                header: StrAllocHeader {
                    _next: prev_first,
                    next_index: sz,
                    _pd: PhantomData,
                },
                array: [0u8; ARRAY_SZ],
            }));

            Ok(&mut self.firstblock.as_mut().unwrap().array[0 .. sz])
        }
    }

    /// Shortcut function to NUL-terminate a string.
    pub fn nulterm(&mut self, s: &str) -> Result<&str, Error>
    {
        let sb = s.as_bytes();
        let buf = self.alloc(sb.len() + 1)?;

        for i in 0 .. sb.len() {
            buf[i] = sb[i];
        }
        buf[sb.len()] = 0;
        Ok(unsafe { str::from_utf8_unchecked(buf) })
    }
}
