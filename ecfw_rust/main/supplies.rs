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

pub use main::power::*;
use main::pins::*;
use main::messages::*;

macro_rules! supply_table {
    (
        $( $name:ident, $kind:tt, $( $arg:expr ),* );* ;
    ) => {
        pub static SUPPLY_TABLE: &'static [&'static(Supply)] = &[
            $( &$name ),*
        ];

        $(
            #[allow(dead_code)]
            pub static $name: $kind = $kind::new( stringify!($name), $( $arg ),* );
        )*
    }
}

supply_table!{

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // VrmSupply (regulator units in the voltage regulator module)
    ///////////////////////////////////////////////////////////////////////////////////////////////
    // supply name
    // |                    dependencies
    // |                    |                           supply ID
    // |                    |                           |   Discharge pin
    // |                    |                           |   |                   Discharge time (ms)
    BUCK_5VA,   VrmSupply,  &[],                        1,  Some((&DISCH_5VA,   72));
    BUCK_5VB,   VrmSupply,  &[],                        2,  Some((&DISCH_5VB,   72));
    BUCK_3VA,   VrmSupply,  &[],                        3,  Some((&DISCH_3VA,   36));
    BUCK_3VB,   VrmSupply,  &[],                        4,  None;
    INV_N12,    VrmSupply,  &[&BUCK_5VA, &BUCK_5VB],    5,  None;

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // GpioSwitchedSupply (supplies enabled by a GPIO)
    ///////////////////////////////////////////////////////////////////////////////////////////////
    // supply name
    //                              dependencies
    //                                                  GPIO control
    //                                                                  Enable/disable time (ms)
    //                                                                      Discharge pin
    //                                                                                          Dsch
    //                                                                                          time
    //                                                                                          (ms)
    LDO_S3,     GpioSwitchedSupply, &[&BUCK_1V5],       &EN_V75REF,     1,  None;
    LDO_S0,     GpioSwitchedSupply, &[&LDO_S3],         &EN_V75,        1,  None;
    BUCK_1V5,   GpioSwitchedSupply, &[&BUCK_5VB],       &EN_1V5,        10, Some((&DISCH_1V5,   12));
    BUCK_1V2,   GpioSwitchedSupply, &[&BUCK_5VA],       &EN_1V2,        10, Some((&DISCH_1V2,   12));
    SW1,        GpioSwitchedSupply, &[],                &EN_P12V_PCI,   1,  None;
    SW2,        GpioSwitchedSupply, &[&BUCK_5VB],       &EN_P5V_PCI_B,  1,  None;
    SW3,        GpioSwitchedSupply, &[&BUCK_3VB],       &EN_P3V3_S0B,   6,  Some((&DISCH_3VB,   36));
}

pub fn transition_s3_from_s5() -> StdResult {
    try!(BUCK_5VA.up());
    try!(BUCK_5VB.up());

    try!(BUCK_5VB.wait_status(SupplyStatus::Up));
    try!(BUCK_1V5.up());

    try!(BUCK_5VA.wait_status(SupplyStatus::Up));
    try!(BUCK_1V2.up());

    try!(BUCK_1V5.wait_status(SupplyStatus::Up));
    try!(LDO_S3.up());

    try!(BUCK_1V2.wait_status(SupplyStatus::Up));
    try!(LDO_S3.wait_status(SupplyStatus::Up));

    Ok(())
}

pub fn transition_s0_from_s3() -> StdResult {
    try!(BUCK_3VA.up());
    try!(LDO_S0.up());
    try!(INV_N12.up());
    try!(SW1.up());
    try!(SW2.up());
    try!(SW3.up());

    try!(BUCK_3VA.wait_status(SupplyStatus::Up));
    try!(LDO_S0.wait_status(SupplyStatus::Up));
    try!(INV_N12.wait_status(SupplyStatus::Up));
    try!(SW1.wait_status(SupplyStatus::Up));
    try!(SW2.wait_status(SupplyStatus::Up));
    try!(SW3.wait_status(SupplyStatus::Up));
    Ok(())
}

pub fn transition_s3_from_s0() -> StdResult {
    try!(SW3.down());
    try!(SW2.down());
    try!(SW1.down());
    try!(INV_N12.down());
    try!(LDO_S0.down());
    try!(BUCK_3VA.down());

    try!(SW3.wait_status(SupplyStatus::Down));
    try!(SW2.wait_status(SupplyStatus::Down));
    try!(SW1.wait_status(SupplyStatus::Down));
    try!(INV_N12.wait_status(SupplyStatus::Down));
    try!(LDO_S0.wait_status(SupplyStatus::Down));
    try!(BUCK_3VA.wait_status(SupplyStatus::Down));
    Ok(())
}

pub fn transition_s5_from_s3() -> StdResult {
    try!(LDO_S3.down());
    try!(BUCK_1V2.down());

    try!(LDO_S3.wait_status(SupplyStatus::Down));
    try!(BUCK_1V5.down());

    try!(BUCK_1V2.wait_status(SupplyStatus::Down));
    try!(BUCK_5VA.down());

    try!(BUCK_1V5.wait_status(SupplyStatus::Down));
    try!(BUCK_5VB.down());

    try!(BUCK_5VA.wait_status(SupplyStatus::Down));
    try!(BUCK_5VB.wait_status(SupplyStatus::Down));
    Ok(())
}
