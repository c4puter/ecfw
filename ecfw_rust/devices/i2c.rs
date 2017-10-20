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

//! Definitions of I2C peripherals

use drivers::i2c::*;
use os::Mutex;

pub static I2C0: I2C = I2C::new(0x40018000 as I2CHandle);
pub static I2C1: I2C = I2C::new(0x4001C000 as I2CHandle);

macro_rules! i2c_table {
    (
        $( $name:ident @ $i2c:ident : $addr:expr ; )*
    ) => {
        $(
            #[allow(dead_code)]
            pub static $name: Mutex<I2CDevice> = Mutex::new(I2CDevice::new(&$i2c, $addr));
        )*
    }
}

i2c_table! {
    U901            @ I2C0:0x20; // PCF8575
    U101            @ I2C0:0x21; // PCF8575
    U801            @ I2C0:0x37; // AS1130
    VRM901          @ I2C0:0x47;
    LM75B_LOGIC     @ I2C0:0x48;
    LM75B_AMBIENT   @ I2C0:0x49;
    SDRAM_SPD       @ I2C0:0x50;
    CDCE913         @ I2C0:0x65; // Clock synthesizer
    PCF8523         @ I2C0:0x68; // RTC
}
