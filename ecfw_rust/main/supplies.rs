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

use main::power::*;
use main::pins::*;
use core::sync::atomic::*;

pub static SUPPLY_TABLE: &'static [&'static(Supply + Sync)] = &[
    &BUCK_5VA,
    &BUCK_5VB,
    &BUCK_3VA,
    &BUCK_3VB,
    &INV_N12,
    &LDO_S3,
    &LDO_S0,
    &BUCK_1V5,
    &BUCK_1V2,
    &SW1,
    &SW2,
];

pub static BUCK_5VA: VrmSupply = VrmSupply::new("BUCK_5VA", &[], 1);
pub static BUCK_5VB: VrmSupply = VrmSupply::new("BUCK_5VB", &[], 2);
pub static BUCK_3VA: VrmSupply = VrmSupply::new("BUCK_3VA", &[], 3);
pub static BUCK_3VB: VrmSupply = VrmSupply::new("BUCK_3VB", &[], 4);
pub static INV_N12: VrmSupply = VrmSupply::new("INV_N12", &[&BUCK_5VA, &BUCK_5VB], 5);

pub static LDO_S3: GpioSwitchedSupply = GpioSwitchedSupply::new(
    "LDO_S3",
    &[&BUCK_1V5],
    &EN_V75REF,
    1);

pub static LDO_S0: GpioSwitchedSupply = GpioSwitchedSupply::new(
    "LDO_S0",
    &[&LDO_S3],
    &EN_V75,
    1);

pub static BUCK_1V5: GpioSwitchedSupply = GpioSwitchedSupply::new(
    "BUCK_1V5",
    &[&BUCK_5VB],
    &EN_1V5,
    10);

pub static BUCK_1V2: GpioSwitchedSupply = GpioSwitchedSupply::new(
    "BUCK_1V2",
    &[&BUCK_5VA],
    &EN_1V2,
    10);

pub static SW1: GpioSwitchedSupply = GpioSwitchedSupply::new(
    "SW1",
    &[],
    &EN_P12V_PCI,
    1);

pub static SW2: GpioSwitchedSupply = GpioSwitchedSupply::new(
    "SW2",
    &[&BUCK_5VB],
    &EN_P5V_PCI_B,
    1);
