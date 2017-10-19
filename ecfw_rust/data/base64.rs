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

//! Conversion to and from base 64

use messages::*;

static BASE64_ENCODE_LUT: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

static BASE64_DECODE_LUT: [u8; 256] = [
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,		//   0..15
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,		//  16..31
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,  62, 255, 255, 255,  63,		//  32..47
	 52,  53,  54,  55,  56,  57,  58,  59,  60,  61, 255, 255, 255,   0, 255, 255,		//  48..63
	255,   0,   1,   2,   3,   4,   5,   6,   7,   8,   9,  10,  11,  12,  13,  14,		//  64..79
	 15,  16,  17,  18,  19,  20,  21,  22,  23,  24,  25, 255, 255, 255, 255, 255,		//  80..95
	255,  26,  27,  28,  29,  30,  31,  32,  33,  34,  35,  36,  37,  38,  39,  40,		//  96..111
	 41,  42,  43,  44,  45,  46,  47,  48,  49,  50,  51, 255, 255, 255, 255, 255,		// 112..127
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,		// 128..143
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];


fn enc_one_symbol(src: u32) -> u8
{
    (*BASE64_ENCODE_LUT)[src as usize]
}

fn dec_one_symbol(src: u8) -> Result<u8, Error>
{
    let val = BASE64_DECODE_LUT[src as usize];

    if val == 255 { Err(ERR_BASE64) } else { Ok(val) }
}

/// Encode a block of data as base64.
///
/// # Arguments
/// - `dest` - buffer to hold the result
/// - `src` - slice of raw data to be encoded as base 64
///
/// # Return
/// - `Ok(n)` - number of bytes written to `dest`
/// - `Err(ERR_STRLEN)` - `dest` is too short
pub fn encode(dest: &mut [u8], src: &[u8]) -> Result<usize, Error>
{
    let mut written = 0usize;

    for i in src.chunks(3) {

        let mut as_bits = i[0] as u32;

        as_bits <<= 8;
        if i.len() > 1 {
            as_bits |= i[1] as u32;
        }

        as_bits <<= 8;
        if i.len() > 2 {
            as_bits |= i[2] as u32;
        }


        let symbol_3 = if i.len() == 3 {
            enc_one_symbol(as_bits & 0x3f)
        } else {
            b'='
        };

        as_bits >>= 6;
        let symbol_2 = if i.len() >= 2 {
            enc_one_symbol(as_bits & 0x3f)
        } else {
            b'='
        };

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

/// Decode a block of data from base64.
///
/// # Arguments
/// - `dest` - buffer to hold the raw result
/// - `src` - slice of data to be decoded from base 64
///
/// # Return
/// - `Ok(n)` - number of bytes written to `dest`
/// - `Err(ERR_BASE64)` - `src` contains invalid data
/// - `Err(ERR_STRLEN)` - `dest` is too short
pub fn decode(dest: &mut [u8], src: &[u8]) -> Result<usize, Error>
{
    let mut written = 0usize;

    if src.len() % 4 != 0 {
        return Err(ERR_BASE64);
    }

    for n in 0 .. (src.len() / 4) {
        let i = &src[n * 4 .. n * 4 + 4];

        let len = if i[2] == b'=' && i[3] == b'=' {
            1
        } else if i[3] == b'=' {
            2
        } else {
            3
        };

        if written + len > dest.len() {
            return Err(ERR_STRLEN);
        }

        let mut as_bits = 0u32;

        for symbol in i {
            as_bits <<= 6;
            as_bits |= dec_one_symbol(*symbol)? as u32;
        }

        dest[written + 0] = ((as_bits & 0xff0000) >> 16) as u8;
        if len > 1 {
            dest[written + 1] = ((as_bits & 0x00ff00) >> 8) as u8;
        }
        if len > 2 {
            dest[written + 2] = (as_bits & 0x0000ff) as u8
        }

        written += len;
    }

    Ok(written)
}
