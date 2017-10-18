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

use core::sync::atomic::*;
use core::cmp::{PartialEq, Eq};
use core::fmt;
use messages;

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
        if $crate::messages::$name.enabled() {
            print!("{:8}: ", $crate::messages::$name.prefix);
            println!( $($values),* );
        }
    }
}

macro_rules! debug_async {
    (
        $name:ident, $( $values:tt ),*
    ) => {
        if $crate::messages::$name.enabled() {
            print_async!("{:8}: ", $crate::messages::$name.prefix);
            print_async!( $($values),* );
            print_async!("\n");
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
