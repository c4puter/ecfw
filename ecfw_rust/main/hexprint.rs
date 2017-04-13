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

use core::str;

/// Print a hex dump.  pub fn hexprint(data: &[u8])
pub fn hexprint(data: &[u8])
{
    // Format:
    //
    // 00000000  01 12 23 34 45 56 67 78  89 9a ab bc cd de ef ff  |................|

    let mut skipping = false;

    for i in 0..data.len() {

        let col = i % 16;

        if col == 0 {
            let is_zero = zero_row(data, i);

            if is_zero && !skipping {
                println!("{:08x}  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|", i);
                println!("*");
            }

            skipping = is_zero;
        }

        if skipping {
            continue;
        }

        if col == 0 {
            print!("{:08x} ", i);
        }

        if col == 8 {
            print!(" ");
        }

        print!(" {:02x}", data[i]);

        if col == 15 || i == (data.len() - 1) {
            for _ in 0..(15 - col) {
                print!("   ");
            }
            if col < 8 {
                print!(" ");
            }
            asciiprint(data, 16 * (i / 16));
        }

    }

    println!("{:08x}", data.len());
}


fn asciiprint(data: &[u8], offset: usize) {
    print!("  |");

    for i in offset..offset+16 {
        if i >= data.len() { break; }
        if isprint(data[i]) {
            print!("{}",
                // safe: isprint
                unsafe{str::from_utf8_unchecked(&data[i..i+1])});
        } else {
            print!(".");
        }
    }

    println!("|");
}


fn isprint(c: u8) -> bool {
    return c > 0x1f && c < 0x7f;
}

fn zero_row(data: &[u8], idata: usize) -> bool
{
    for i in idata..data.len() {
        if data[i] != 0 {
            return false;
        }
        if i >= idata + 16 {
            break;
        }
    }

    return true;
}
