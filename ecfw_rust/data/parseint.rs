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

use messages::*;

/// A string-to-int parser that handles radix specifiers.
///
///     use parseint::ParseInt;
///
///     let s = "-0x1337";
///     let x = i32::parseint(s).unwrap();
///
///     assert_eq!(-4919, x);
///
/// Radices can be specified as follows:
///
/// - `0b`, `0B`: binary
/// - `0o`, `0O`, `0`: octal
/// - `0d`, `0D`: decimal (default)
/// - `0x`, `0X`: hexadecimal

pub trait ParseInt
where
    Self: Sized,
{
    fn parseint(s: &str) -> Result<Self, Error>;
}

fn parsedigit(c: char, radix: u8) -> Result<u8, Error>
{
    match c.to_digit(radix as u32) {
        Some(c) => Ok(c as u8),
        None => Err(ERR_DIGIT),
    }
}

impl ParseInt for u64 {
    fn parseint(s: &str) -> Result<u64, Error>
    {
        #[derive(PartialEq)]
        enum State {
            Radix1,
            Radix2,
            Number,
        }
        let mut state = State::Radix1;
        let mut accumulator: u64 = 0;
        let mut radix: u64 = 10;

        for c in s.chars() {
            if state == State::Radix1 {
                // Leading zero
                if c == '0' {
                    state = State::Radix2;
                    continue;
                } else {
                    state = State::Number;
                }
            }

            if state == State::Radix2 {
                // o, b, d, x
                state = State::Number;
                if c == 'o' || c == 'O' {
                    radix = 8;
                    continue;
                } else if c == 'b' || c == 'B' {
                    radix = 2;
                    continue;
                } else if c == 'd' || c == 'D' {
                    radix = 10;
                    continue;
                } else if c == 'x' || c == 'X' {
                    radix = 16;
                    continue;
                } else {
                    radix = 8;
                }
            }

            if state == State::Number {
                let parsed = parsedigit(c, radix as u8)?;
                let limit = (u64::max_value() - parsed as u64) / radix;
                if accumulator > limit {
                    return Err(ERR_NRANGE);
                }
                accumulator = accumulator * radix + parsed as u64;
            }
        }

        Ok(accumulator)
    }
}

impl ParseInt for i64 {
    fn parseint(s: &str) -> Result<i64, Error>
    {
        // Here's the tricky bit. i64 parseint is the one that performs the
        // conversion between signed and unsigned. Watch ranges!

        let firstchar = match s.chars().nth(0) {
            None => return Ok(0),
            Some(c) => c,
        };

        let absval = match firstchar {
            '+' | '-' => u64::parseint(s.split_at(1).1)?,
            _ => u64::parseint(s)?,
        };

        if firstchar == '-' {
            if absval == u64::max_value() / 2 + 1 {
                return Ok(i64::min_value());
            } else if absval <= u64::max_value() / 2 {
                return Ok(-(absval as i64));
            } else {
                return Err(ERR_NRANGE);
            }
        } else {
            if absval > (i64::max_value() as u64) {
                return Err(ERR_NRANGE);
            } else {
                return Ok(absval as i64);
            }
        }
    }
}

impl ParseInt for u32 {
    fn parseint(s: &str) -> Result<u32, Error>
    {
        match u64::parseint(s)? {
            n if n <= (u32::max_value() as u64) => Ok(n as u32),
            _ => Err(ERR_NRANGE),
        }
    }
}
impl ParseInt for u16 {
    fn parseint(s: &str) -> Result<u16, Error>
    {
        match u64::parseint(s)? {
            n if n <= (u16::max_value() as u64) => Ok(n as u16),
            _ => Err(ERR_NRANGE),
        }
    }
}
impl ParseInt for u8 {
    fn parseint(s: &str) -> Result<u8, Error>
    {
        match u64::parseint(s)? {
            n if n <= (u8::max_value() as u64) => Ok(n as u8),
            _ => Err(ERR_NRANGE),
        }
    }
}

impl ParseInt for i32 {
    fn parseint(s: &str) -> Result<i32, Error>
    {
        match i64::parseint(s)? {
            n
                if n >= (i32::min_value() as i64) &&
                   n <= (i32::max_value() as i64) => Ok(n as i32),
            _ => Err(ERR_NRANGE),
        }
    }
}
impl ParseInt for i16 {
    fn parseint(s: &str) -> Result<i16, Error>
    {
        match i64::parseint(s)? {
            n
                if n >= (i16::min_value() as i64) &&
                   n <= (i16::max_value() as i64) => Ok(n as i16),
            _ => Err(ERR_NRANGE),
        }
    }
}
impl ParseInt for i8 {
    fn parseint(s: &str) -> Result<i8, Error>
    {
        match i64::parseint(s)? {
            n
                if n >= (i8::min_value() as i64) &&
                   n <= (i8::max_value() as i64) => Ok(n as i8),
            _ => Err(ERR_NRANGE),
        }
    }
}
