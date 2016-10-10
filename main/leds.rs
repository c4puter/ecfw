/*
 * The MIT License (MIT)
 * Copyright (c) 2016 Chris Pavlina
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

#![no_std]
extern crate twi;
extern crate smutex;
extern crate ledmatrix;
use twi::TwiDevice;
use ledmatrix::Led;

pub static U801: TwiDevice = TwiDevice {
    twi: twi::twi0,
    addr: 0x37,
    mutex: smutex::StaticMutex{locked: false},
};

macro_rules! led_table {
    (
        $( $name:ident , $addr:expr );* ;
    ) => {
        pub static LED_TABLE: &'static [&'static Led] = &[
            $( &$name ),*
        ];

        $( pub static $name: Led = Led { addr: $addr, name: stringify!($name) }; )*
    }
}

led_table!{
    ///////////////////////////////////
    // Power section
    P12V_PCI_R,     0x75;
    P12V_PCI_G,     0x44;

    P5V_PCI_A_R,    0x54;
    P5V_PCI_A_G,    0x74;

    P5V_PCI_B_R,    0x52;
    P5V_PCI_B_G,    0x42;

    P3V3_PCI_A_R,   0x53;
    P3V3_PCI_A_G,   0x43;

    P3V3_PCI_B_R,   0x50;
    P3V3_PCI_B_G,   0x40;

    N12V_PCI_R,     0x51;
    N12V_PCI_G,     0x41;

    P3V3_STBY_R,    0x25;
    P3V3_STBY_G,    0x34;

    P3V3_AUX_R,     0x23;
    P3V3_AUX_G,     0x33;

    P3V3_LOGIC_R,   0x72;
    P3V3_LOGIC_G,   0x32;

    P1V5_LOGIC_R,   0x22;
    P1V5_LOGIC_G,   0x73;

    P1V2_LOGIC_R,   0x20;
    P1V2_LOGIC_G,   0x30;

    PV75_TERM_R,    0x21;
    PV75_TERM_G,    0x31;

    ///////////////////////////////////
    // Boot sequence
    ECFW_R,         0x04;
    ECFW_G,         0x14;

    POWER_R,        0x03;
    POWER_G,        0x31;

    CARD_R,         0x01;
    CARD_G,         0x11;

    BIT_R,          0x02;
    BIT_BRIDGE_G,   0x12;
    BIT_CPU0_G,     0xA2;
    BIT_CPU1_G,     0xB2;

    MEM_R,          0x70;
    MEM_G,          0x10;

    RUN_R,          0x00;
    RUN_G,          0x71;
    UPDOG_G,        0xB1;

    ///////////////////////////////////
    // Uncommitted
    UNC0_R,         0x95;
    UNC0_G,         0x85;
    UNC1_R,         0x94;
    UNC1_G,         0x84;
    UNC2_R,         0x92;
    UNC2_G,         0x82;
    UNC3_R,         0x93;
    UNC3_G,         0x83;
    UNC4_R,         0x90;
    UNC4_G,         0x80;
    UNC5_R,         0x91;
    UNC5_G,         0x81;
}
