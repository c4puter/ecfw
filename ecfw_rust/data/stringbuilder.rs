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

use messages::*;
use core::str;
use core::slice;
use core::mem;
use alloc::boxed::Box;
pub const MAXLEN: usize = 1024;

/// Helper to build a string from pieces inside a Box.
///
/// MAXLEN is chosen as exactly 1kB; several other possibly heap-
/// allocated types in this system are also that size. This is done to reduce
/// heap fragmentation.
pub struct StringBuilder {
    inner: Box<[u8]>,
    len: usize,
}

impl StringBuilder {
    pub fn new() -> StringBuilder {
        StringBuilder {
            inner: Box::new([0u8; MAXLEN]),
            len: 0,
        }
    }

    /// Append a string. Return Err(ERR_STRLEN) if the string does not fit in
    /// the already allocated box. On error, the contents of this StringBuilder
    /// were not modified.
    pub fn append(&mut self, s: &str) -> StdResult {
        let sb = s.as_bytes();
        let count = sb.len();
        let avail = MAXLEN - self.len;

        if avail >= count {
            self.inner[self.len..self.len+count].clone_from_slice(sb);
            self.len += count;
            Ok(())
        } else {
            Err(ERR_STRLEN)
        }
    }

    /// Append a single character, encoded as UTF-8. Behaves the same as
    /// append().
    pub fn append_char(&mut self, c: char) -> StdResult {
        let mut buf = [0u8; 4];
        self.append(c.encode_utf8(&mut buf))
    }

    /// Get the constructed string as a box, consuming the builder.
    pub fn into_box(self) -> Box<str> {
        unsafe {
            let raw = Box::into_raw(self.inner);
            let slice = slice::from_raw_parts_mut(raw as *mut u8, self.len);
            let newbox = Box::from_raw(slice);
            mem::transmute::<Box<[u8]>, Box<str>>(newbox)
        }
    }

    /// Get the constructed string as a borrowed reference.
    pub fn as_ref(&self) -> &str {
        unsafe{mem::transmute::<&[u8], &str>(&self.inner[0..self.len])}
    }

    /// Get the current length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Truncate to the given length.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
        }
    }

    /// Advanced: mutably borrow the internal buffer, for passing to C
    /// functions. Afterward, you MUST call fix_length() to update the
    /// internal length counter.
    ///
    /// If `partial` is true, only borrow the part that has not been
    /// filled yet (so the C function can append).
    pub unsafe fn as_mut_ref(&mut self, partial: bool) -> &mut [u8]
    {
        let r = if partial {
            &mut self.inner[self.len..]
        } else {
            &mut self.inner
        };

        // Zero-terminate the buffer being given so fix_length() doesn't
        // resurrect truncated strings if nothing is appended
        r[0] = 0;
        r
    }

    /// Advanced: recalculate length after a C function appended to the
    /// buffer via as_mut_ref(). This stops at a NUL, assuming a valid
    /// C string was added, but skips over anything already accounted
    /// for (so any existing NULs in the buffer are not disturbed).
    pub unsafe fn fix_length(&mut self)
    {
        let start = self.len;
        for i in start..MAXLEN {
            if self.inner[i] == 0 {
                break;
            } else {
                self.len += 1;
            }
        }
    }
}
