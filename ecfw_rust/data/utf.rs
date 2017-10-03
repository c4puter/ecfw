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

use messages::*;
use core::char;

/// Read UTF-16LE data into a UTF-8 buffer. Processes the entire length, but
/// returns the number of consecutive nonzero bytes written.
///
/// Errors on invalid codepoints, including orphaned surrogates. Asserts that
/// dest is long enough (must be at least src.len()*2).
///
/// Warning - this is not necessarily standard-compliant: it does not guarantee
/// errors on invalid conditions, only when it can't figure out what to do.
/// In particular, a single isolated high surrogate will be silently dropped
/// (whereas a single isolated low surrogate will cause complaint).
pub fn read_utf16le_into_utf8(dest: &mut [u8], src: &[u8]) -> Result<usize, Error>
{
    assert!(dest.len() >= src.len() * 2);

    let mut prev_surrogate = 0u32;
    let mut codepoint = 0u32;
    let mut idest = 0usize;
    let mut first_zero: Option<usize> = None;

    for isrc in 0..src.len() {
        if isrc % 2 == 0 {
            codepoint = src[isrc] as u32;
            continue;
        }

        codepoint |= (src[isrc] as u32) << 8;

        if codepoint >= 0xD800 && codepoint <= 0xDBFF {
            // high surrogate
            prev_surrogate = codepoint;
            continue;
        }
        if codepoint >= 0xDC00 && codepoint <= 0xDFFF {
            // low surrogate
            if prev_surrogate == 0 {
                return Err(ERR_UTF16_ORPHAN);
            } else {
                codepoint = 0x10000 +
                    ((prev_surrogate & 0x03ff) << 10) +
                    (codepoint & 0x03ff);
            }
        }
        if codepoint > 0x10FFFF {
            return Err(ERR_CODEPOINT);
        }

        if codepoint == 0 && first_zero.is_none() {
            first_zero = Some(idest);
        }

        idest += write_one_utf8(dest, idest, codepoint);
    }

    match first_zero {
        Some(x) => { Ok(x) },
        None    => { Ok(idest) },
    }
}

/// Write a single codepoint into a buffer, returning the number of bytes written
pub fn write_one_utf8(dest: &mut [u8], idest: usize, codepoint: u32) -> usize
{
    let c = unsafe{char::from_u32_unchecked(codepoint)};
    let destlen = dest.len();
    let s = c.encode_utf8(&mut dest[idest..destlen]);
    s.len()
}
