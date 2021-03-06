// c4puter embedded controller firmware
// Copyright (C) 2017 Chris Pavlina
//
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

//! Generic COM (RS232/CDC-like) trait and helpers

use os;
use core::fmt;

static OUT_MUTEX: os::Mutex<()> = os::Mutex::new(());

/// COM (RS232/CDC-like) interface.
pub trait Com {
    /// Get one byte, returning `None` immediately if none is available.
    fn getc(&self) -> Option<u8>;

    /// Put one byte, blocking if necessary.
    fn putc(&self, c: u8);

    /// Put one byte immediately. This may depend on ISR-pumped buffers if
    /// necessary but should not depend on OS tasks.
    fn putc_async(&self, c: u8);

    /// Put one byte, dropping it silently if the transmit buffers are full.
    fn putc_nowait(&self, c: u8);

    /// Flush the output buffer if possible, returning whether it was done
    /// (not all interfaces need to support this).
    fn flush_output(&self) -> bool;

    /// Get one byte, blocking until one is available.
    ///
    /// # Arguments
    ///
    /// * `yield_task` - whether the task should yield to other tasks until
    ///     input is ready (most polite) or loop tightly (best throughput).
    fn getc_blocking(&self, yield_task: bool) -> u8
    {
        loop {
            if let Some(x) = self.getc() {
                return x;
            }

            if yield_task {
                os::yield_task();
            }
        }
    }
}

/// Get the next byte available from any of the given interfaces.
///
/// # Arguments
///
/// * `interfaces` - a list of interfaces to check
/// * `yield_task` - whether the task should yield to other tasks until
///     input is ready (most polite) or loop tightly (best throughput).
pub fn getc_any_blocking(interfaces: &[&Com], yield_task: bool) -> u8
{
    loop {
        for i in interfaces {
            if let Some(x) = i.getc() {
                return x;
            }
        }

        if yield_task {
            os::yield_task();
        }
    }
}

struct ComWriter<'a> {
    com: &'a Com,
}

struct ComWriterAsync<'a> {
    com: &'a Com,
}

struct ComWriterNoWait<'a> {
    com: &'a Com,
}

impl<'a> fmt::Write for ComWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        for c in s.as_bytes() {
            if *c == b'\n' {
                self.com.putc(b'\r');
            }
            self.com.putc(*c);
        }
        return Ok(());
    }
}

impl<'a> fmt::Write for ComWriterAsync<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        for c in s.as_bytes() {
            if *c == b'\n' {
                self.com.putc_async(b'\r');
            }
            self.com.putc_async(*c);
        }
        return Ok(());
    }
}

impl<'a> fmt::Write for ComWriterNoWait<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        for c in s.as_bytes() {
            if *c == b'\n' {
                self.com.putc_nowait(b'\r');
            }
            self.com.putc_nowait(*c);
        }
        return Ok(());
    }
}

/// Print immediately to an interface.
pub fn print_async(interface: &Com, args: fmt::Arguments)
{
    let mut cw = ComWriterAsync { com: interface };
    fmt::write(&mut cw, args).unwrap();
}

/// Print to an interface one call at a time, guarded by a mutex.
///
/// Will be interrupted by any simultaneous asynchronous prints.
pub fn print(interface: &Com, args: fmt::Arguments)
{
    let _lock = OUT_MUTEX.lock();
    let mut cw = ComWriter { com: interface };
    fmt::write(&mut cw, args).unwrap();
}

/// Print to all listed interfaces one call at a time, guarded by a mutex.
///
/// Mutex is acquired individually for each print to each interface. Will be
/// interrupted by any simultaneous asynchronous prints.
pub fn print_all(interfaces: &[&Com], args: fmt::Arguments)
{
    for i in interfaces {
        let _lock = OUT_MUTEX.lock();
        let mut cw = ComWriterNoWait { com: *i };
        fmt::write(&mut cw, args).unwrap();
    }
}
