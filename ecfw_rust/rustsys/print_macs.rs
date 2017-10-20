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

//! Print macros

/// Write to a COM interface.
///
/// Equivalent to [`println_to!`] except that a newline is not
/// printed, and equivalent to the [`print!`] macro except that you can
/// choose the destination interface.
///
/// # Arguments
/// - `interface` - one of the devices implementing [Com]
/// - `fmt...` - format string and arguments
///
/// [Com]: drivers/com/trait.Com.html
/// [`println_to!`]: macro.println_to.html
/// [`print!`]: macro.print.html
#[macro_export]
macro_rules! print_to {
    ($interface:expr, $($arg:tt)*) => (
        $crate::drivers::com::print($interface, format_args!($($arg)*))
    );
}

/// Write to a COM interface, with newline.
///
/// Equivalent to [`print_to!`] macro except that a newline is printed,
/// and equivalent to [`println!`] except that you can choose the
/// destination interface.
///
/// # Arguments
/// - `interface` - one of the devices implementing [Com]
/// - `[fmt...]` - format string and arguments (optional, will print a blank
///                line if format string is omitted)
///
/// [Com]: drivers/com/trait.Com.html
/// [`print_to!`]: macro.print_to.html
/// [`println!`]: macro.println.html
#[macro_export]
macro_rules! println_to {
    ($int:expr) => (
        print_to!($int, "\n")
    );
    ($int:expr, $fmt:expr) => (
        print_to!($int, concat!($fmt, "\n"))
    );
    ($int:expr, $fmt:expr, $($arg:tt)*) => (
        print_to!($int, concat!($fmt, "\n"), $($arg)*)
    );
}

/// Write to each of a list of COM interfaces.
///
/// # Arguments
/// - `interfaces` - a `&[&Com]` listing all the interfaces to print to
/// - `fmt...` - format string and arguments
#[macro_export]
macro_rules! print_all {
    ($interfaces:expr, $($arg:tt)*) => (
        $crate::drivers::com::print_all($interfaces, format_args!($($arg)*))
    );
}

/// Write to each of a list of COM interfaces, with newline.
///
/// # Arguments
/// - `interfaces` - a `&[&Com]` listing all the interfaces to print to
/// - `[fmt...]` - format string and arguments (optional, will print a blank
///                line if format string is omitted)
#[macro_export]
macro_rules! println_all {
    ($ints:expr) => (
        print_all!($ints, "\n")
    );
    ($ints:expr, $fmt:expr) => (
        print_all!($ints, concat!($fmt, "\n"))
    );
    ($ints:expr, $fmt:expr, $($arg:tt)*) => (
        print_all!($ints, concat!($fmt, "\n"), $($arg)*)
    );
}

/// Write asynchronously to a COM interface.
///
/// Equivalent to [`print_to!`] except that the interface will not
/// be locked for the duration of the print. This can be used in cases where
/// unlocking would be impossible (e.g. panic handlers where the task scheduler
/// has been halted), but beware that if you use it while other tasks are
/// running, multiple outputs may become intermixed.
///
/// # Arguments
/// - `interface` - one of the devices implementing [Com]
/// - `fmt...` - format string and arguments
///
/// [Com]: drivers/com/trait.Com.html
/// [`print_to!`]: macro.print_to.html
#[macro_export]
macro_rules! print_to_async {
    ($interface:expr, $($arg:tt)*) => (
        $crate::drivers::com::print_async($interface, format_args!($($arg)*))
    );
}

/// Write to the standard debug interfaces.
///
/// Equivalent to [`print_all!`] with a target list including [COMUSART] and
/// [COMCDC].
///
/// # Arguments
/// - `fmt...` - format string and arguments
///
/// [`print_all!`]: macro.print_all.html
/// [COMUSART]: drivers/com_usart/static.COMUSART.html
/// [COMCDC]: drivers/com_cdc/static.COMCDC.html
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        print_all!(
            &[&$crate::devices::COMUSART as &$crate::drivers::com::Com,
              &$crate::devices::COMCDC as &$crate::drivers::com::Com],
              $($arg)*)
    );
}

/// Write to the standard debug interfaces, with newline.
///
/// Equivalent to [`println_all!`] with a target list including [COMUSART] and
/// [COMCDC].
///
/// # Arguments
/// - `[fmt...]` - format string and arguments (optional, will print a blank
///                line if format string is omitted)
///
/// [`println_all!`]: macro.println_all.html
/// [COMUSART]: drivers/com_usart/static.COMUSART.html
/// [COMCDC]: drivers/com_cdc/static.COMCDC.html
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => (
        println_all!(
            &[&$crate::devices::COMUSART as &$crate::drivers::com::Com,
              &$crate::devices::COMCDC as &$crate::drivers::com::Com],
              $($arg)*)
    );
}

/// Write asynchronously to the standard RS232 debug interface.
///
/// Equivalent to [`print_to_async!`] with a target of [COMUSART]. [COMCDC] is
/// not included because it is not always available, and thus not appropriate
/// for e.g. panic handlers.
///
/// # Arguments
/// - `fmt...` - format string and arguments
///
/// [`print_to_async!`]: macro.print_to_async.html
/// [COMUSART]: drivers/com_usart/static.COMUSART.html
/// [COMCDC]: drivers/com_cdc/static.COMCDC.html
#[macro_export]
macro_rules! print_async {
    ($($arg:tt)*) => (
        print_to_async!(&$crate::devices::COMUSART, $($arg)*)
    );
}
