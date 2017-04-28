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
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
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

static BASE64_ENCODE_LUT: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn enc_one_symbol(src: u32) -> u8 { (*BASE64_ENCODE_LUT)[src as usize] }

fn dec_one_symbol(src: u8) -> Result<u8,Error>
{
    match src {
        b'A' ... b'Z' => Ok(src - b'A'),
        b'a' ... b'z' => Ok(src - b'a' + 26),
        b'0' ... b'9' => Ok(src - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        b'=' => Ok(0),
        _ => Err(ERR_BASE64)
    }
}

/// Encode a block of data as base64. Return number of base64 characters written.
pub fn encode(dest: &mut [u8], src: &[u8]) -> Result<usize,Error>
{
    let mut written = 0usize;

    for i in src.chunks(3) {

        let mut as_bits = i[0] as u32;

        as_bits <<= 8;
        if i.len() > 1 { as_bits |= i[1] as u32; }

        as_bits <<= 8;
        if i.len() > 2 { as_bits |= i[2] as u32; }


        let symbol_3 = if i.len() == 3 { enc_one_symbol(as_bits & 0x3f) } else { b'=' };

        as_bits >>= 6;
        let symbol_2 = if i.len() >= 2 { enc_one_symbol(as_bits & 0x3f) } else { b'=' };

        as_bits >>= 6;
        let symbol_1 = enc_one_symbol(as_bits & 0x3f);

        as_bits >>= 6;
        let symbol_0 = enc_one_symbol(as_bits & 0x3f);

        if dest.len() - written < 4 {
            return Err(ERR_STRLEN);
        }

        dest[written + 0] = symbol_0;
        dest[written + 1] = symbol_1;
        dest[written + 2] = symbol_2;
        dest[written + 3] = symbol_3;
        written += 4;
    }

    Ok(written)
}

/// Decode a block of data from base64. Return number of bytes written.
pub fn decode(dest: &mut [u8], src: &[u8]) -> Result<usize,Error>
{
    let mut written = 0usize;

    if src.len() % 4 != 0 {
        return Err(ERR_BASE64);
    }

    for i in src.chunks(4) {
        let len =
            if i[2] == b'=' && i[3] == b'=' { 1 }
            else if i[3] == b'='            { 2 }
            else                            { 3 };

        if written + len >= dest.len() {
            return Err(ERR_STRLEN);
        }

        let mut as_bits = 0u32;

        for symbol in i {
            as_bits <<= 6;
            as_bits |= try!(dec_one_symbol(*symbol)) as u32;
        }

        dest[written + 0] = ((as_bits & 0xff0000) >> 16) as u8;
        if len > 1 { dest[written + 1] = ((as_bits & 0x00ff00) >> 8) as u8; }
        if len > 2 { dest[written + 2] =  (as_bits & 0x0000ff) as u8  }

        written += len;
    }

    Ok(written)
}
