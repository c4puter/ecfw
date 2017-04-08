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

pub struct DebugClass {
    pub name: &'static str,
    pub prefix: &'static str,
    enabled: AtomicBool,
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
                enabled: AtomicBool::new($default),
            };
        )*
    };
}

debug_table! {
    DEBUG_SYSMAN: "sysman", true;
    DEBUG_PWRBTN: "pwrbtn", false;
}

macro_rules! debug {
    (
        $name:ident, $( $values:tt ),*
    ) => {
        if $crate::main::debug::$name.enabled() {
            print!("{}: ", $crate::main::debug::$name.prefix);
            println!( $($values),* );
        }
    }
}

pub fn debug_set(name: &str, enabled: bool) -> bool
{
    for &dbg in DEBUG_TABLE {
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
