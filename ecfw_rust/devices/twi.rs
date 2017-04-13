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
