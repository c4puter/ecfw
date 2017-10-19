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

use core::str;

/// Print a hex dump.  pub fn hexprint(data: &[u8])
pub fn hexprint(data: &[u8])
{
    // Format:
    //
    // 00000000  01 12 23 34 45 56 67 78  89 9a ab bc cd de ef ff  |................|

    let mut skipping = false;

    for i in 0 .. data.len() {

        let col = i % 16;

        if col == 0 {
            let is_zero = zero_row(data, i);

            if is_zero && !skipping {
                println!(
                    "{:08x}  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  \
                     |................|",
                    i
                );
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
            for _ in 0 .. (15 - col) {
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


fn asciiprint(data: &[u8], offset: usize)
{
    print!("  |");

    for i in offset .. offset + 16 {
        if i >= data.len() {
            break;
        }
        if isprint(data[i]) {
            print!(
                "{}",
                // safe: isprint
                unsafe { str::from_utf8_unchecked(&data[i .. i + 1]) }
            );
        } else {
            print!(".");
        }
    }

    println!("|");
}


fn isprint(c: u8) -> bool
{
    return c > 0x1f && c < 0x7f;
}

fn zero_row(data: &[u8], idata: usize) -> bool
{
    for i in idata .. data.len() {
        if data[i] != 0 {
            return false;
        }
        if i >= idata + 16 {
            break;
        }
    }

    return true;
}
