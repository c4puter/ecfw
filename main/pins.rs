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
extern crate gpio;
use gpio::*;

pub const PIN_TABLE: &'static [&'static Gpio] = &[
    // Integrated GPIOs
    &BRIDGE_SUSP,
    &CARD,
    &CARDEN,
    &CPU_SUSP,
    &FAN_PWM,
    &FAN_TACH,
    &FPGA_CCLK,
    &FPGA_DATA,
    &FPGA_DONE0,
    &FPGA_DONE1,
    &FPGA_DONE2,
    &FPGA_INIT,
    &PANELINT,
    &PCIM66EN,
    &PCIPME,
    &PCIRST,
    &PROGRAM0,
    &PROGRAM1,
    &PROGRAM2,
    &PRSNT1_0,
    &PRSNT1_1,
    &PRSNT1_2,
    &PRSNT1_3,
    &PRSNT2_0,
    &PRSNT2_1,
    &PRSNT2_2,
    &PRSNT2_3,
    &REQ,
    &RTCINT,
    &SDRAM_EVENT,
    &SDRAM_RST,
    &USB_VBSENSE,
    &VREFEN,

    // IO expander: U101
    &ACT_LED,
    &DEBUG_BOOT,
    &FLASH_BTN,
    &FORCE_POWER,
    &LOW_SPEED,
    &MERGE_SERIAL,
    &POWER_BTN,
    &POWER_LED,
    &SINGLE_CPU,
    &SPEAKER,
    &UNC_SW_0,
    &UNC_SW_1,
    &UNC_SW_2,

    // IO expander: U901
    &DISCH_1V5,
    &DISCH_3VA,
    &DISCH_3VB,
    &DISCH_5VA,
    &DISCH_5VB,
    &EN_1V2,
    &EN_1V5,
    &EN_5V_PCI_B,
    &EN_P12V_PCI,
    &EN_P3V3_S0B,
    &EN_SAFETY,
    &EN_V75,
    &EN_V75REF,
];

pub const USB_VBSENSE: SamGpio = SamGpio {
    name:   "USB_VBSENSE",  default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 0,             invert: false   };
pub const PRSNT2_2: SamGpio = SamGpio {
    name:   "PRSNT2_2",     default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 1,             invert: true };
pub const PRSNT1_2: SamGpio = SamGpio {
    name:   "PRSNT1_2",     default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 2,             invert: true };
pub const FPGA_INIT: SamGpio = SamGpio {
    name:   "FPGA_INIT",    default: false,     mode: Mode::Output,
    port:   PIOA,           pin: 5,             invert: true };
pub const FPGA_DONE2: SamGpio = SamGpio {
    name:   "FPGA_DONE2",   default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 6,             invert: false };
pub const FPGA_DONE1: SamGpio = SamGpio {
    name:   "FPGA_DONE1",   default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 7,             invert: false };
pub const FPGA_DONE0: SamGpio = SamGpio {
    name:   "FPGA_DONE0",   default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 8,             invert: false };
pub const PROGRAM2: SamGpio = SamGpio {
    name:   "PROGRAM2",     default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 10,            invert: false };
pub const PROGRAM1: SamGpio = SamGpio {
    name:   "PROGRAM1",     default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 9,             invert: false };
pub const PROGRAM0: SamGpio = SamGpio {
    name:   "PROGRAM0",     default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 11,            invert: false };
pub const FPGA_DATA: SamGpio = SamGpio {
    name:   "FPGA_DATA",    default: false,     mode: Mode::Output,
    port:   PIOA,           pin: 13,            invert: false   };
pub const FPGA_CCLK: SamGpio = SamGpio {
    name:   "FPGA_CCLK",    default: false,     mode: Mode::Output,
    port:   PIOA,           pin: 14,            invert: false   };
pub const FAN_TACH: SamGpio = SamGpio {
    name:   "FAN_TACH",     default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 15,            invert: false   };
pub const REQ: SamGpio = SamGpio {
    name:   "REQ",          default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 16,            invert: false   };
pub const CARD: SamGpio = SamGpio {
    name:   "CARD",         default: false,     mode: Mode::Input,
    port:   PIOA,           pin: 20,            invert: true    };
pub const FAN_PWM: SamGpio = SamGpio {
    name:   "FAN_PWM",      default: false,     mode: Mode::Output,
    port:   PIOA,           pin: 22,            invert: false   };
pub const PRSNT1_3: SamGpio = SamGpio {
    name:   "PRSNT1_3",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 9,             invert: true };
pub const PRSNT2_3: SamGpio = SamGpio {
    name:   "PRSNT2_3",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 10,            invert: true };
pub const PRSNT1_1: SamGpio = SamGpio {
    name:   "PRSNT1_1",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 16,            invert: true };
pub const PANELINT: SamGpio = SamGpio {
    name:   "PANELINT",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 17,            invert: true };
pub const SDRAM_EVENT: SamGpio = SamGpio {
    name:   "SDRAM_EVENT",  default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 18,            invert: true };
pub const PRSNT2_1: SamGpio = SamGpio {
    name:   "PRSNT2_1",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 19,            invert: true };
pub const PRSNT2_0: SamGpio = SamGpio {
    name:   "PRSNT2_0",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 20,            invert: true };
pub const PRSNT1_0: SamGpio = SamGpio {
    name:   "PRSNT1_0",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 21,            invert: true };
pub const PCIRST: SamGpio = SamGpio {
    name:   "PCIRST",       default: false,     mode: Mode::Output,
    port:   PIOC,           pin: 22,            invert: true };
pub const PCIPME: SamGpio = SamGpio {
    name:   "PCIPME",       default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 23,            invert: true };
pub const PCIM66EN: SamGpio = SamGpio {
    name:   "PCIM66EN",     default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 24,            invert: false };
pub const RTCINT: SamGpio = SamGpio {
    name:   "RTCINT",       default: false,     mode: Mode::Input,
    port:   PIOC,           pin: 25,            invert: true };
pub const CPU_SUSP: SamGpio = SamGpio {
    name:   "CPU_SUSP",     default: true,      mode: Mode::Output,
    port:   PIOC,           pin: 27,            invert: false };
pub const CARDEN: SamGpio = SamGpio {
    name:   "CARDEN",       default: false,     mode: Mode::Output,
    port:   PIOC,           pin: 28,            invert: true };
pub const BRIDGE_SUSP: SamGpio = SamGpio {
    name:   "BRIDGE_SUSP",  default: true,      mode: Mode::Output,
    port:   PIOC,           pin: 31,            invert: false };
pub const VREFEN: SamGpio = SamGpio {
    name:   "VREFEN",       default: false,     mode: Mode::Output,
    port:   PIOB,           pin: 0,             invert: false };
pub const SDRAM_RST: SamGpio = SamGpio {
    name:   "SDRAM_RST",    default: true,      mode: Mode::Output,
    port:   PIOB,           pin: 13,            invert: true };

const OUTPUTS_U101: u16 = 0x0030;

pub const POWER_LED: PcfGpio = PcfGpio {
    name:   "POWER_LED",    default: false,
    addr:   0x21,           pin: 15,            invert: true,   outputs: OUTPUTS_U101 };
pub const ACT_LED: PcfGpio = PcfGpio {
    name:   "ACT_LED",      default: false,
    addr:   0x21,           pin: 14,            invert: true,   outputs: OUTPUTS_U101 };
pub const POWER_BTN: PcfGpio = PcfGpio {
    name:   "POWER_BTN",    default: false,
    addr:   0x21,           pin: 16,            invert: true,   outputs: OUTPUTS_U101 };
pub const FLASH_BTN: PcfGpio = PcfGpio {
    name:   "FLASH_BTN",    default: false,
    addr:   0x21,           pin: 16,            invert: true,   outputs: OUTPUTS_U101 };
pub const SPEAKER: PcfGpio = PcfGpio {
    name:   "SPEAKER",      default: false,
    addr:   0x21,           pin: 10,            invert: true,   outputs: OUTPUTS_U101 };
pub const MERGE_SERIAL: PcfGpio = PcfGpio {
    name:   "MERGE_SERIAL", default: false,
    addr:   0x21,           pin: 7,             invert: true,   outputs: OUTPUTS_U101 };
pub const DEBUG_BOOT: PcfGpio = PcfGpio {
    name:   "DEBUG_BOOT",   default: false,
    addr: 0x21,             pin: 6,             invert: true,   outputs: OUTPUTS_U101 };
pub const SINGLE_CPU: PcfGpio = PcfGpio {
    name:   "SINGLE_CPU",   default: false,
    addr: 0x21,             pin: 5,             invert: true,   outputs: OUTPUTS_U101 };
pub const FORCE_POWER: PcfGpio = PcfGpio {
    name:   "FORCE_POWER",  default: false,
    addr: 0x21,             pin: 4,             invert: true,   outputs: OUTPUTS_U101 };
pub const LOW_SPEED: PcfGpio = PcfGpio {
    name:   "LOW_SPEED",    default: false,
    addr: 0x21,             pin: 3,             invert: true,   outputs: OUTPUTS_U101 };
pub const UNC_SW_2: PcfGpio = PcfGpio {
    name:   "UNC_SW_2",     default: false,
    addr: 0x21,             pin: 2,             invert: true,   outputs: OUTPUTS_U101 };
pub const UNC_SW_1: PcfGpio = PcfGpio {
    name:   "UNC_SW_1",     default: false,
    addr: 0x21,             pin: 1,             invert: true,   outputs: OUTPUTS_U101 };
pub const UNC_SW_0: PcfGpio = PcfGpio {
    name:   "UNC_SW_0",     default: false,
    addr: 0x21,             pin: 0,             invert: true,   outputs: OUTPUTS_U101 };

const OUTPUTS_U901: u16 = 0xcfff;

pub const EN_V75: PcfGpio = PcfGpio {
    name:   "EN_V75",       default: false,
    addr: 0x20,             pin: 0,             invert: false,  outputs: OUTPUTS_U901 };
pub const EN_V75REF: PcfGpio = PcfGpio {
    name:   "EN_V75REF",    default: false,
    addr: 0x20,             pin: 1,             invert: false,  outputs: OUTPUTS_U901 };
pub const EN_1V2: PcfGpio = PcfGpio {
    name:   "EN_1V2",       default: false,
    addr: 0x20,             pin: 2,             invert: false,  outputs: OUTPUTS_U901 };
pub const EN_1V5: PcfGpio = PcfGpio {
    name:   "EN_1V5",       default: false,
    addr: 0x20,             pin: 3,             invert: false,  outputs: OUTPUTS_U901 };
pub const EN_SAFETY: PcfGpio = PcfGpio {
    name:   "EN_SAFETY",    default: true,
    addr: 0x20,             pin: 6,             invert: false,  outputs: OUTPUTS_U901 };
pub const DISCH_1V5: PcfGpio = PcfGpio {
    name:   "DISCH_1V5",    default: true,
    addr: 0x20,             pin: 7,             invert: true,   outputs: OUTPUTS_U901 };
pub const DISCH_3VA: PcfGpio = PcfGpio {
    name:   "DISCH_3VA",    default: true,
    addr: 0x20,             pin: 10,            invert: true,   outputs: OUTPUTS_U901 };
pub const DISCH_3VB: PcfGpio = PcfGpio {
    name:   "DISCH_3VB",    default: true,
    addr: 0x20,             pin: 11,            invert: true,   outputs: OUTPUTS_U901 };
pub const EN_P3V3_S0B: PcfGpio = PcfGpio {
    name:   "EN_P3V3_S0B",  default: false,
    addr: 0x20,             pin: 12,            invert: true,   outputs: OUTPUTS_U901 };
pub const DISCH_5VA: PcfGpio = PcfGpio {
    name:   "DISCH_5VA",    default: true,
    addr: 0x20,             pin: 13,            invert: false,  outputs: OUTPUTS_U901 };
pub const DISCH_5VB: PcfGpio = PcfGpio {
    name:   "DISCH_5VB",    default: true,
    addr: 0x20,             pin: 14,            invert: false,  outputs: OUTPUTS_U901 };
pub const EN_5V_PCI_B: PcfGpio = PcfGpio {
    name:   "EN_5V_PCI_B",  default: false,
    addr: 0x20,             pin: 15,            invert: true,   outputs: OUTPUTS_U901 };
pub const EN_P12V_PCI: PcfGpio = PcfGpio {
    name:   "EN_P12V_PCI",  default: false,
    addr: 0x20,             pin: 16,            invert: false,  outputs: OUTPUTS_U901 };
pub const DISCH_1V2: PcfGpio = PcfGpio {
    name:   "DISCH_1V2",    default: true,
    addr: 0x20,             pin: 17,            invert: true,   outputs: OUTPUTS_U901 };


