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

use drivers::twi::*;
use os::Mutex;

pub static TWI0: Twi = Twi::new(0x40018000 as TwiHandle);
pub static TWI1: Twi = Twi::new(0x4001C000 as TwiHandle);

macro_rules! twi_table {
    (
        $( $name:ident @ $twi:ident : $addr:expr ; )*
    ) => {
        $(
            #[allow(dead_code)]
            pub static $name: Mutex<TwiDevice> = Mutex::new(TwiDevice::new(&$twi, $addr));
        )*
    }
}

twi_table! {
    U901            @ TWI0:0x20; // PCF8575
    U101            @ TWI0:0x21; // PCF8575
    U801            @ TWI0:0x37; // AS1130
    VRM901          @ TWI0:0x47;
    LM75B_LOGIC     @ TWI0:0x48;
    LM75B_AMBIENT   @ TWI0:0x49;
    SDRAM_SPD       @ TWI0:0x50;
    CDCE913         @ TWI0:0x65; // Clock synthesizer
    PCF8523         @ TWI0:0x68; // RTC
}
