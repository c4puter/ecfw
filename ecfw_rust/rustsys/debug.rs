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

use core::sync::atomic::*;
use core::cmp::{PartialEq, Eq};
use core::fmt;
use main::messages;

pub struct DebugClass {
    pub name: &'static str,
    pub prefix: &'static str,
    pub enabled: AtomicBool,
}

macro_rules! debug_table {
    (
        $( $name:ident : $prefix:expr, $default:expr ; )*
    ) => {
        #[allow(unused)]
        pub static DEBUG_TABLE: &'static [&'static DebugClass] = &[
            $( &$name ),*
        ];

        $(
            #[allow(dead_code)]
            pub static $name: DebugClass = DebugClass {
                name: stringify!($name),
                prefix: $prefix,
                enabled: ::core::sync::atomic::AtomicBool::new($default),
            };
        )*
    };
}

macro_rules! debug {
    (
        $name:ident, $( $values:tt ),*
    ) => {
        if $crate::main::messages::$name.enabled() {
            print!("{:8}: ", $crate::main::messages::$name.prefix);
            println!( $($values),* );
        }
    }
}

macro_rules! debug_async {
    (
        $name:ident, $( $values:tt ),*
    ) => {
        if $crate::main::messages::$name.enabled() {
            print_async!("{:8}: ", $crate::main::messages::$name.prefix);
            println_async!( $($values),* );
        }
    }
}

pub fn debug_set(name: &str, enabled: bool) -> bool
{
    for &dbg in messages::DEBUG_TABLE {
        if *(dbg.name) == *name {
            dbg.enable(enabled);
            return true;
        }
    }
    false
}

impl DebugClass {
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn enable(&self, v: bool) {
        self.enabled.store(v, Ordering::SeqCst);
    }
}

#[derive(Copy,Clone)]
pub struct Error {
    pub message: &'static str,
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool
    {
        let self_msg = self.message as *const str;
        let other_msg = other.message as *const str;
        self_msg == other_msg
    }
}

impl Eq for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "Error: {}", self.message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}", self.message)
    }
}

macro_rules! error_table {
    ( $( $name:ident : $message:expr ; )* ) =>
    {
        #[allow(unused)]
        pub static ERROR_TABLE: &'static [&'static Error] = &[ $( &$name ),* ];

        $(
            #[allow(dead_code)]
            pub static $name: Error = Error { message: $message };
        )*
    }
}

pub type StdResult = Result<(), Error>;
