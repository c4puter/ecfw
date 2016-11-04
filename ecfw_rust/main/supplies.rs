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

macro_rules! supply_table {
    (
        $( $name:ident, $kind:tt, $( $arg:expr ),* );* ;
    ) => {
        pub static SUPPLY_TABLE: &'static [&'static(Supply + Sync)] = &[
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

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // VirtualSupply (virtual named rails)
    ///////////////////////////////////////////////////////////////////////////////////////////////
    P12V_PCI,           VirtualSupply,  &[&SW1];
    P5V_PCI_A,          VirtualSupply,  &[&BUCK_5VA];
    P5V_PCI_B,          VirtualSupply,  &[&SW2];
    P3V3_PCI_A,         VirtualSupply,  &[&BUCK_3VA, &P1V2_CORE];
    P3V3_PCI_B,         VirtualSupply,  &[&SW3, &P1V2_CORE];
    N12V_PCI,           VirtualSupply,  &[&INV_N12];
    P1V2_CORE,          VirtualSupply,  &[&BUCK_1V2];
    P1V5_BRIDGE,        VirtualSupply,  &[&BUCK_1V5, &P1V2_CORE];
    P3V3_BRIDGE,        VirtualSupply,  &[&SW3, &P1V2_CORE];
    PV75_SDRAM_VTT,     VirtualSupply,  &[&LDO_S0, &P1V5_BRIDGE];
    PV75_SDRAM_VREF,    VirtualSupply,  &[&LDO_S3, &P1V5_BRIDGE];
    P3V3_CPU,           VirtualSupply,  &[&SW3, &P1V2_CORE];
    P3V3_AUX,           VirtualSupply,  &[&BUCK_3VA, &P1V2_CORE];
    P3V3_STBY,          VirtualSupply,  &[&BUCK_3VB];

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // VirtualSupply (power states)
    ///////////////////////////////////////////////////////////////////////////////////////////////
    S0,                 VirtualSupply,  &[&P12V_PCI, &P5V_PCI_A, &P5V_PCI_B,
                                          &N12V_PCI, &P3V3_PCI_A, &P3V3_PCI_B,
                                          &P1V2_CORE, &P1V5_BRIDGE, &P3V3_BRIDGE,
                                          &PV75_SDRAM_VTT, &PV75_SDRAM_VREF,
                                          &P3V3_CPU, &P3V3_AUX, &P3V3_STBY];
    S3,                 VirtualSupply,  &[&P1V2_CORE, &P1V5_BRIDGE,
                                          &PV75_SDRAM_VREF, &P3V3_STBY];
    S5,                 VirtualSupply,  &[&P3V3_STBY];
}
