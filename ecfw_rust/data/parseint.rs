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

use messages::*;

/// A str->int parser that handles radix specifiers.
///
///     extern crate parseint;
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
/// - `0d`, `0D`, ``: decimal
/// - `0x`, `0X`: hexadecimal

pub trait ParseInt where Self: Sized {
    fn parseint(s: &str) -> Result<Self,Error>;
}

fn parsedigit(c: char, radix: u8) -> Result<u8,Error> {
    match c.to_digit(radix as u32) {
        Some(c) => Ok(c as u8),
        None => Err(ERR_DIGIT)
    }
}

impl ParseInt for u64 {
    fn parseint(s: &str) -> Result<u64,Error> {
        #[derive(PartialEq)]
        enum State {
            Radix1,
            Radix2,
            Number
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
                let parsed = try!(parsedigit(c, radix as u8));
                let limit = (u64::max_value() - parsed as u64) / radix;
                if accumulator > limit {
                    return Err(ERR_NRANGE)
                }
                accumulator = accumulator * radix + parsed as u64;
            }
        }

        Ok(accumulator)
    }
}

impl ParseInt for i64 {
    fn parseint(s: &str) -> Result<i64,Error> {
        // Here's the tricky bit. i64 parseint is the one that performs the
        // conversion between signed and unsigned. Watch ranges!

        let firstchar = match s.chars().nth(0) {
            None => return Ok(0),
            Some(c) => c,
        };

        let absval = match firstchar {
            '+' | '-' => try!(u64::parseint(s.split_at(1).1)),
            _ => try!(u64::parseint(s))
        };

        if firstchar == '-' {
            if absval == u64::max_value() / 2 + 1 {
                return Ok(i64::min_value());
            } else if absval <= u64::max_value() / 2 {
                return Ok(-(absval as i64));
            } else {
                return Err(ERR_NRANGE)
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
    fn parseint(s: &str) -> Result<u32,Error> {
        match try!(u64::parseint(s)) {
            n if n <= (u32::max_value() as u64) => Ok(n as u32),
            _ => Err(ERR_NRANGE)
        }
    }
}
impl ParseInt for u16 {
    fn parseint(s: &str) -> Result<u16,Error> {
        match try!(u64::parseint(s)) {
            n if n <= (u16::max_value() as u64) => Ok(n as u16),
            _ => Err(ERR_NRANGE)
        }
    }
}
impl ParseInt for u8 {
    fn parseint(s: &str) -> Result<u8,Error> {
        match try!(u64::parseint(s)) {
            n if n <= (u8::max_value() as u64) => Ok(n as u8),
            _ => Err(ERR_NRANGE)
        }
    }
}

impl ParseInt for i32 {
    fn parseint(s: &str) -> Result<i32,Error> {
        match try!(i64::parseint(s)) {
            n if n >= (i32::min_value() as i64) && n <= (i32::max_value() as i64) => Ok(n as i32),
            _ => Err(ERR_NRANGE)
        }
    }
}
impl ParseInt for i16 {
    fn parseint(s: &str) -> Result<i16,Error> {
        match try!(i64::parseint(s)) {
            n if n >= (i16::min_value() as i64) && n <= (i16::max_value() as i64) => Ok(n as i16),
            _ => Err(ERR_NRANGE)
        }
    }
}
impl ParseInt for i8 {
    fn parseint(s: &str) -> Result<i8,Error> {
        match try!(i64::parseint(s)) {
            n if n >= (i8::min_value() as i64) && n <= (i8::max_value() as i64) => Ok(n as i8),
            _ => Err(ERR_NRANGE)
        }
    }
}

