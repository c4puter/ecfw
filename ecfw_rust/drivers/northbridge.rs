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

//! Northbridge data bus interface.

use os::Mutex;
use messages::*;
use devices;
use rustsys::rust_support::{disable_irq, enable_irq};

use asf_pio::Pio;
use core::ptr::{read_volatile, write_volatile};

static NB_MUTEX: Mutex<()> = Mutex::new(());

//////////////////////////////////////
// Fast, direct GPIO access functions.
//
// Similar functions are declared in pio.h, but defined in pio.c, and without
// LTO we end up with actual function calls. Too slow for this.

macro_rules! def_get {
    ($name:ident, $member:ident) => (
        fn $name(p: *const Pio) -> u32
        {
            unsafe {
                read_volatile(&((*p).$member) as *const _)
            }
        }
    )
}

macro_rules! def_set {
    ($name:ident, $member:ident) => (
        fn $name(p: *mut Pio, mask: u32)
        {
            unsafe {
                write_volatile(&mut((*p).$member) as *mut _, mask)
            }
        }
    )
}

const PIO: *mut Pio = 0x400E1200 as *mut Pio;
const CLK_BM: u32 = 1 << 14;
const NRD_BM: u32 = 1 << 11;
const START_BM: u32 = 1 << 8;
const NWAIT_BM: u32 = 1 << 13;

def_get!(get_output_write_status, PIO_OWSR);
def_get!(get_pins, PIO_PDSR);

def_set!(enable_output_write, PIO_OWER);
def_set!(disable_output_write, PIO_OWDR);
def_set!(write_pins, PIO_ODSR);
def_set!(set_pins, PIO_SODR);
def_set!(clear_pins, PIO_CODR);
def_set!(pins_output, PIO_OER);
def_set!(pins_input, PIO_ODR);

pub struct Northbridge {}

impl Northbridge {
    pub const fn new() -> Northbridge
    {
        Northbridge {}
    }

    fn send_addr(&self, dest_addr: u64)
    {
        let addr0 = ((dest_addr & 0x00000000FF) >> 0) as u32;
        let addr1 = ((dest_addr & 0x000000FF00) >> 8) as u32;
        let addr2 = ((dest_addr & 0x0000FF0000) >> 16) as u32;
        let addr3 = ((dest_addr & 0x00FF000000) >> 24) as u32;
        let addr4 = ((dest_addr & 0x0F00000000) >> 32) as u32;

        set_pins(PIO, NRD_BM | START_BM);
        pins_output(PIO, 0xFF);

        while get_pins(PIO) & NWAIT_BM == 0 {}

        unsafe { disable_irq() };

        let owsr = get_output_write_status(PIO);
        disable_output_write(PIO, 0xFFFFFFFF);
        enable_output_write(PIO, CLK_BM | 0xFF | START_BM);

        write_pins(PIO, addr0 | START_BM);
        set_pins(PIO, CLK_BM);

        write_pins(PIO, addr1);
        set_pins(PIO, CLK_BM);

        write_pins(PIO, addr2);
        set_pins(PIO, CLK_BM);

        write_pins(PIO, addr3);
        set_pins(PIO, CLK_BM);

        write_pins(PIO, addr4);
        set_pins(PIO, CLK_BM);

        disable_output_write(PIO, 0xFFFFFFFF);
        enable_output_write(PIO, owsr);

        unsafe { enable_irq() };
    }

    fn send_data(&self, data: u32)
    {
        let data0 = ((data & 0x000000FF) >> 0) as u32;
        let data1 = ((data & 0x0000FF00) >> 8) as u32;
        let data2 = ((data & 0x00FF0000) >> 16) as u32;
        let data3 = ((data & 0xFF000000) >> 24) as u32;

        set_pins(PIO, NRD_BM);
        pins_output(PIO, 0xFF);

        unsafe { disable_irq() };

        let owsr = get_output_write_status(PIO);
        disable_output_write(PIO, 0xFFFFFFFF);
        enable_output_write(PIO, CLK_BM | 0xFF);

        write_pins(PIO, data0);
        set_pins(PIO, CLK_BM);

        write_pins(PIO, data1);
        set_pins(PIO, CLK_BM);

        write_pins(PIO, data2);
        set_pins(PIO, CLK_BM);

        write_pins(PIO, data3);
        set_pins(PIO, CLK_BM);

        disable_output_write(PIO, 0xFFFFFFFF);
        enable_output_write(PIO, owsr);

        unsafe { enable_irq() };

        while get_pins(PIO) & NWAIT_BM == 0 {}
    }

    fn get_data(&self) -> u32
    {
        pins_input(PIO, 0xFF);
        clear_pins(PIO, NRD_BM);

        clear_pins(PIO, CLK_BM);
        set_pins(PIO, CLK_BM);
        while get_pins(PIO) & NWAIT_BM == 0 {}

        let data0 = get_pins(PIO) & 0xFF;
        clear_pins(PIO, CLK_BM);
        set_pins(PIO, CLK_BM);

        let data1 = get_pins(PIO) & 0xFF;
        clear_pins(PIO, CLK_BM);
        set_pins(PIO, CLK_BM);

        let data2 = get_pins(PIO) & 0xFF;
        clear_pins(PIO, CLK_BM);
        set_pins(PIO, CLK_BM);

        let data3 = get_pins(PIO) & 0xFF;

        (data0 << 0) | (data1 << 8) | (data2 << 16) | (data3 << 24)
    }

    fn finish_read(&self)
    {
        set_pins(PIO, NRD_BM);
        pins_output(PIO, 0xFF);
        while get_pins(PIO) & NWAIT_BM == 0 {}
    }

    // Write a block of data into the northbridge. Destination is word-addressed
    pub fn poke(&self, dest_addr: u64, src: &[u32]) -> StdResult
    {
        let mutlock = devices::FPGAS[0].proglock();

        if mutlock.is_none() {
            return Err(ERR_FPGA_BEFORE_BITSTREAM);
        }

        let _lock = NB_MUTEX.lock();

        for i in 0 .. src.len() {
            let dest_addr_i = dest_addr + (i as u64);

            if i == 0 || (dest_addr_i & 0xFF) == 0 {
                // Northbridge interface has autoincrement over the last octet
                // of the address. If we would overflow this autoincrement, we
                // must retransmit the address.
                self.send_addr(dest_addr_i);
            }

            self.send_data(src[i]);
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

        for i in 0 .. dest.len() {
            let src_addr_i = src_addr + (i as u64);

            if i == 0 || (src_addr_i & 0xFF) == 0 {
                // Northbridge interface has autoincrement over the last octet
                // of the address. If we would overflow this autoincrement, we
                // must retransmit the address.
                if i != 0 {
                    self.finish_read();
                }
                self.send_addr(src_addr_i);
            }

            dest[i] = self.get_data();
        }

        Ok(())
    }
}
