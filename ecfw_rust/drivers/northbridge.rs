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

use os::Mutex;
use messages::*;
use devices;
extern crate bindgen_mcu;

static NB_MUTEX: Mutex<()> = Mutex::new(());

pub struct Northbridge {}

impl Northbridge {
    pub const fn new() -> Northbridge { Northbridge{} }

    // Write a block of data into the northbridge. Destination is word-addressed
    pub fn poke(&self, dest_addr: u64, src: &[u32]) -> StdResult
    {
        let mutlock = devices::FPGAS[0].proglock();

        if mutlock.is_none() {
            return Err(ERR_FPGA_BEFORE_BITSTREAM);
        }

        let _lock = NB_MUTEX.lock();
        let n = src.len();

        unsafe {
            bindgen_mcu::northbridge_poke(dest_addr, src.as_ptr(), n as u32);
        }

        Ok(())
    }

    // Read a block of data from the northbridge. Source is word-addressed
    pub fn peek(&self, dest: &mut [u32], src_addr: u64) -> StdResult
    {
        let mutlock = devices::FPGAS[0].proglock();

        if mutlock.is_none() {
            return Err(ERR_FPGA_BEFORE_BITSTREAM);
        }

        let _lock = NB_MUTEX.lock();
        let n = dest.len();

        unsafe {
            bindgen_mcu::northbridge_peek(dest.as_mut_ptr(), src_addr, n as u32);
        }

        Ok(())
    }
}
