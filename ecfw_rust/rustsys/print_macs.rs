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

#[macro_export]
macro_rules! print_to {
    ($interface:expr, $($arg:tt)*) => ($crate::drivers::com::print($interface, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_to {
    ($interface:expr, $($arg:tt)*) => ($crate::drivers::com::println($interface, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_all {
    ($interfaces:expr, $($arg:tt)*) => ($crate::drivers::com::print_all($interfaces, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_all {
    ($interfaces:expr, $($arg:tt)*) => ($crate::drivers::com::println_all($interfaces, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_to_async {
    ($interface:expr, $($arg:tt)*) => ($crate::drivers::com::print_async($interface, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print { ($($arg:tt)*) => (print_all!(
            &[&$crate::devices::COMUSART as &$crate::drivers::com::Com,
              &$crate::devices::COMCDC as &$crate::drivers::com::Com],
              $($arg)*)) }

#[macro_export]
macro_rules! println { ($($arg:tt)*) => (println_all!(
            &[&$crate::devices::COMUSART as &$crate::drivers::com::Com,
              &$crate::devices::COMCDC as &$crate::drivers::com::Com],
              $($arg)*)) }

#[macro_export]
macro_rules! print_async { ($($arg:tt)*) => (print_to_async!(&$crate::devices::COMUSART, $($arg)*)) }
