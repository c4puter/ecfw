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

//! Definitions of power supplies and sequencing functions

pub use drivers::power::*;
use devices::pins::*;
use messages::*;

macro_rules! supply_table {
    (
        $( $name:ident, $kind:tt, $( $arg:expr ),* );* ;
    ) => {
        pub static SUPPLY_TABLE: &[&(Supply)] = &[
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

pub fn transition_s3_from_s5() -> StdResult
{
    BUCK_5VB.up()?;
    BUCK_5VA.up()?;

    BUCK_5VB.wait_status(SupplyStatus::Up)?;
    BUCK_1V5.up()?;

    BUCK_5VA.wait_status(SupplyStatus::Up)?;
    BUCK_1V2.up()?;

    BUCK_1V5.wait_status(SupplyStatus::Up)?;
    LDO_S3.up()?;

    BUCK_1V2.wait_status(SupplyStatus::Up)?;
    LDO_S3.wait_status(SupplyStatus::Up)?;

    Ok(())
}

pub fn transition_s0_from_s3() -> StdResult
{
    BUCK_3VA.up()?;
    LDO_S0.up()?;
    INV_N12.up()?;
    SW1.up()?;
    SW2.up()?;
    SW3.up()?;

    BUCK_3VA.wait_status(SupplyStatus::Up)?;
    LDO_S0.wait_status(SupplyStatus::Up)?;
    INV_N12.wait_status(SupplyStatus::Up)?;
    SW1.wait_status(SupplyStatus::Up)?;
    SW2.wait_status(SupplyStatus::Up)?;
    SW3.wait_status(SupplyStatus::Up)?;
    Ok(())
}

pub fn transition_s3_from_s0() -> StdResult
{
    SW3.down()?;
    SW2.down()?;
    SW1.down()?;
    INV_N12.down()?;
    LDO_S0.down()?;
    BUCK_3VA.down()?;

    SW3.wait_status(SupplyStatus::Down)?;
    SW2.wait_status(SupplyStatus::Down)?;
    SW1.wait_status(SupplyStatus::Down)?;
    INV_N12.wait_status(SupplyStatus::Down)?;
    LDO_S0.wait_status(SupplyStatus::Down)?;
    BUCK_3VA.wait_status(SupplyStatus::Down)?;
    Ok(())
}

pub fn transition_s5_from_s3() -> StdResult
{
    LDO_S3.down()?;
    BUCK_1V2.down()?;

    LDO_S3.wait_status(SupplyStatus::Down)?;
    BUCK_1V5.down()?;

    BUCK_1V2.wait_status(SupplyStatus::Down)?;
    BUCK_5VA.down()?;

    BUCK_1V5.wait_status(SupplyStatus::Down)?;
    BUCK_5VB.down()?;

    BUCK_5VA.wait_status(SupplyStatus::Down)?;
    BUCK_5VB.wait_status(SupplyStatus::Down)?;
    Ok(())
}
