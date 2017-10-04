/*
 * c4puter embedded controller firmware
 * Copyright (C) 2017 Chris Pavlina
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

use drivers::gpio::*;
use drivers::ledmatrix::LedGpio;
use drivers::gpio::Mode::*;
use devices::twi::{U101,U901};
use devices::MATRIX;

macro_rules! pin_table {
    (
        $( $name:ident, $kind:tt, $( $key:ident => $data:expr ),* );* ;
    ) => {
        pub static PIN_TABLE: &[&(Gpio + Sync)] = &[
            $( &$name ),*
        ];

        $(
            #[allow(dead_code)]
            pub static $name: $kind = $kind {
                name: stringify!($name),
                $($key : $data),* ,
                // Pending: figure out why the fuck this doesn't work in the macro expansion
                // .. $kind::default()
            };
        )*
    }
}

// PCF8575
const OUTPUTS_U101: u16 = 0x0030;

// PCF8575
const OUTPUTS_U901: u16 = 0xcfff;

pin_table!{

    ///////////////////////////////////
    // MCU GPIOs: FPGA interface, PCI, interrupts, etc

    BRIDGE_SUSP,        SamGpio, port => PIOC, pin => 31, mode => Output, default => false, invert => false;
    CARDEN,             SamGpio, port => PIOC, pin => 28, mode => Output, default => false, invert => true;
    CARD,               SamGpio, port => PIOA, pin => 20, mode => Input,  default => false, invert => true;
    CPU_SUSP,           SamGpio, port => PIOC, pin => 27, mode => Output, default => false, invert => false;
    FAN_PWM,            SamGpio, port => PIOA, pin => 22, mode => Output, default => false, invert => false;
    FAN_TACH,           SamGpio, port => PIOA, pin => 15, mode => Input,  default => false, invert => false;
    FPGA_CCLK,          SamGpio, port => PIOA, pin => 14, mode => PerA,   default => false, invert => false;
    FPGA_DATA,          SamGpio, port => PIOA, pin => 13, mode => PerA,   default => false, invert => false;
    FPGA_DONE0,         SamGpio, port => PIOA, pin =>  8, mode => Input,  default => false, invert => false;
    FPGA_DONE1,         SamGpio, port => PIOA, pin =>  7, mode => Input,  default => false, invert => false;
    FPGA_DONE2,         SamGpio, port => PIOA, pin =>  6, mode => Input,  default => false, invert => false;
    FPGA_INIT,          SamGpio, port => PIOA, pin =>  5, mode => Pullup, default => false, invert => true;
    PANELINT,           SamGpio, port => PIOC, pin => 17, mode => Pullup, default => false, invert => true;
    PCIM66EN,           SamGpio, port => PIOC, pin => 24, mode => Input,  default => false, invert => false;
    PCIPME,             SamGpio, port => PIOC, pin => 23, mode => Input,  default => false, invert => true;
    PCIRST,             SamGpio, port => PIOC, pin => 22, mode => Output, default => false, invert => true;
    FPGA_PROG0,         SamGpio, port => PIOA, pin => 11, mode => Output, default => false, invert => true;
    FPGA_PROG1,         SamGpio, port => PIOA, pin =>  9, mode => Output, default => false, invert => true;
    FPGA_PROG2,         SamGpio, port => PIOA, pin => 10, mode => Output, default => false, invert => true;
    PRSNT1_0,           SamGpio, port => PIOC, pin => 21, mode => Input,  default => false, invert => true;
    PRSNT1_1,           SamGpio, port => PIOC, pin => 16, mode => Input,  default => false, invert => true;
    PRSNT1_2,           SamGpio, port => PIOA, pin =>  2, mode => Input,  default => false, invert => true;
    PRSNT1_3,           SamGpio, port => PIOC, pin =>  9, mode => Input,  default => false, invert => true;
    PRSNT2_0,           SamGpio, port => PIOC, pin => 20, mode => Input,  default => false, invert => true;
    PRSNT2_1,           SamGpio, port => PIOC, pin => 19, mode => Input,  default => false, invert => true;
    PRSNT2_2,           SamGpio, port => PIOA, pin =>  1, mode => Input,  default => false, invert => true;
    PRSNT2_3,           SamGpio, port => PIOC, pin => 10, mode => Input,  default => false, invert => true;
    REQ,                SamGpio, port => PIOA, pin => 16, mode => Input,  default => false, invert => false;
    RS232_RX,           SamGpio, port => PIOA, pin => 21, mode => PerA,   default => false, invert => false;
    RS232_TX,           SamGpio, port => PIOA, pin => 22, mode => PerA,   default => false, invert => false;
    RTCINT,             SamGpio, port => PIOC, pin => 25, mode => Input,  default => false, invert => true;
    SDRAM_EVENT,        SamGpio, port => PIOC, pin => 18, mode => Input,  default => false, invert => true;
    SDRAM_RST,          SamGpio, port => PIOB, pin => 13, mode => Output, default => false, invert => true;
    TWI0_SCL,           SamGpio, port => PIOA, pin => 4,  mode => PerA,   default => false, invert => false;
    TWI0_SDA,           SamGpio, port => PIOA, pin => 3,  mode => PerA,   default => false, invert => false;
    USB_VBSENSE,        SamGpio, port => PIOA, pin =>  0, mode => Input,  default => false, invert => false;
    VREFEN,             SamGpio, port => PIOB, pin =>  0, mode => Output, default => false, invert => false;

    // SD card interface
    MCDA0,              SamGpio, port => PIOA, pin => 30, mode => PerC,   default => false, invert => false;
    MCDA1,              SamGpio, port => PIOA, pin => 31, mode => PerC,   default => false, invert => false;
    MCDA2,              SamGpio, port => PIOA, pin => 26, mode => PerC,   default => false, invert => false;
    MCDA3,              SamGpio, port => PIOA, pin => 27, mode => PerC,   default => false, invert => false;
    MCCK,               SamGpio, port => PIOA, pin => 29, mode => PerC,   default => false, invert => false;
    MCCDA,              SamGpio, port => PIOA, pin => 28, mode => PerC,   default => false, invert => false;

    ///////////////////////////////////
    // Interface IO expander: switches, speaker, some LEDs

    ACT_LED,            PcfGpio, dev => &U101, pin => 14, outputs => OUTPUTS_U101, default => false, invert => true;
    DEBUG_BOOT,         PcfGpio, dev => &U101, pin =>  6, outputs => OUTPUTS_U101, default => false, invert => true;
    FLASH_BTN,          PcfGpio, dev => &U101, pin => 17, outputs => OUTPUTS_U101, default => false, invert => true;
    FORCE_POWER,        PcfGpio, dev => &U101, pin =>  4, outputs => OUTPUTS_U101, default => false, invert => true;
    LOW_SPEED,          PcfGpio, dev => &U101, pin =>  3, outputs => OUTPUTS_U101, default => false, invert => true;
    MERGE_SERIAL,       PcfGpio, dev => &U101, pin =>  7, outputs => OUTPUTS_U101, default => false, invert => true;
    POWER_BTN,          PcfGpio, dev => &U101, pin => 16, outputs => OUTPUTS_U101, default => false, invert => true;
    POWER_LED,          PcfGpio, dev => &U101, pin => 15, outputs => OUTPUTS_U101, default => false, invert => true;
    SINGLE_CPU,         PcfGpio, dev => &U101, pin =>  5, outputs => OUTPUTS_U101, default => false, invert => true;
    SPEAKER,            PcfGpio, dev => &U101, pin => 10, outputs => OUTPUTS_U101, default => false, invert => true;
    UNC_SW_0,           PcfGpio, dev => &U101, pin =>  0, outputs => OUTPUTS_U101, default => false, invert => true;
    UNC_SW_1,           PcfGpio, dev => &U101, pin =>  1, outputs => OUTPUTS_U101, default => false, invert => true;
    UNC_SW_2,           PcfGpio, dev => &U101, pin =>  2, outputs => OUTPUTS_U101, default => false, invert => true;

    DISCH_1V2,          PcfGpio, dev => &U901, pin => 17, outputs => OUTPUTS_U901, default => true,  invert => true;
    DISCH_1V5,          PcfGpio, dev => &U901, pin =>  7, outputs => OUTPUTS_U901, default => true,  invert => true;
    DISCH_3VA,          PcfGpio, dev => &U901, pin => 10, outputs => OUTPUTS_U901, default => true,  invert => true;
    DISCH_3VB,          PcfGpio, dev => &U901, pin => 11, outputs => OUTPUTS_U901, default => true,  invert => true;
    DISCH_5VA,          PcfGpio, dev => &U901, pin => 13, outputs => OUTPUTS_U901, default => true,  invert => false;
    DISCH_5VB,          PcfGpio, dev => &U901, pin => 14, outputs => OUTPUTS_U901, default => true,  invert => false;
    EN_1V2,             PcfGpio, dev => &U901, pin =>  2, outputs => OUTPUTS_U901, default => false, invert => true;
    EN_1V5,             PcfGpio, dev => &U901, pin =>  3, outputs => OUTPUTS_U901, default => false, invert => true;
    EN_P5V_PCI_B,       PcfGpio, dev => &U901, pin => 15, outputs => OUTPUTS_U901, default => false, invert => true;
    EN_P12V_PCI,        PcfGpio, dev => &U901, pin => 16, outputs => OUTPUTS_U901, default => false, invert => false;
    EN_P3V3_S0B,        PcfGpio, dev => &U901, pin => 12, outputs => OUTPUTS_U901, default => false, invert => true;
    EN_SAFETY,          PcfGpio, dev => &U901, pin =>  6, outputs => OUTPUTS_U901, default => true,  invert => false;
    EN_V75,             PcfGpio, dev => &U901, pin =>  0, outputs => OUTPUTS_U901, default => false, invert => false;
    EN_V75REF,          PcfGpio, dev => &U901, pin =>  1, outputs => OUTPUTS_U901, default => false, invert => false;

    ///////////////////////////////////
    // Power LEDs

    P12V_PCI_R,         LedGpio, addr => 0x75, matrix => &MATRIX;
    P12V_PCI_G,         LedGpio, addr => 0x44, matrix => &MATRIX;
    P5V_PCI_A_R,        LedGpio, addr => 0x54, matrix => &MATRIX;
    P5V_PCI_A_G,        LedGpio, addr => 0x74, matrix => &MATRIX;

    P5V_PCI_B_R,        LedGpio, addr => 0x52, matrix => &MATRIX;
    P5V_PCI_B_G,        LedGpio, addr => 0x42, matrix => &MATRIX;

    P3V3_PCI_A_R,       LedGpio, addr => 0x53, matrix => &MATRIX;
    P3V3_PCI_A_G,       LedGpio, addr => 0x43, matrix => &MATRIX;

    P3V3_PCI_B_R,       LedGpio, addr => 0x50, matrix => &MATRIX;
    P3V3_PCI_B_G,       LedGpio, addr => 0x40, matrix => &MATRIX;

    N12V_PCI_R,         LedGpio, addr => 0x51, matrix => &MATRIX;
    N12V_PCI_G,         LedGpio, addr => 0x41, matrix => &MATRIX;

    P3V3_STBY_R,        LedGpio, addr => 0x25, matrix => &MATRIX;
    P3V3_STBY_G,        LedGpio, addr => 0x34, matrix => &MATRIX;

    P3V3_AUX_R,         LedGpio, addr => 0x23, matrix => &MATRIX;
    P3V3_AUX_G,         LedGpio, addr => 0x33, matrix => &MATRIX;

    P3V3_LOGIC_R,       LedGpio, addr => 0x72, matrix => &MATRIX;
    P3V3_LOGIC_G,       LedGpio, addr => 0x32, matrix => &MATRIX;

    P1V5_LOGIC_R,       LedGpio, addr => 0x22, matrix => &MATRIX;
    P1V5_LOGIC_G,       LedGpio, addr => 0x73, matrix => &MATRIX;

    P1V2_LOGIC_R,       LedGpio, addr => 0x20, matrix => &MATRIX;
    P1V2_LOGIC_G,       LedGpio, addr => 0x30, matrix => &MATRIX;

    PV75_TERM_R,        LedGpio, addr => 0x21, matrix => &MATRIX;
    PV75_TERM_G,        LedGpio, addr => 0x31, matrix => &MATRIX;

    ///////////////////////////////////
    // Boot sequence LEDs
    ECFW_R,             LedGpio, addr => 0x04, matrix => &MATRIX;
    ECFW_G,             LedGpio, addr => 0x14, matrix => &MATRIX;

    POWER_R,            LedGpio, addr => 0x03, matrix => &MATRIX;
    POWER_G,            LedGpio, addr => 0x13, matrix => &MATRIX;

    CARD_R,             LedGpio, addr => 0x01, matrix => &MATRIX;
    CARD_G,             LedGpio, addr => 0x11, matrix => &MATRIX;

    BIT_R,              LedGpio, addr => 0x02, matrix => &MATRIX;
    BIT_BRIDGE_G,       LedGpio, addr => 0x12, matrix => &MATRIX;
    BIT_CPU0_G,         LedGpio, addr => 0xA2, matrix => &MATRIX;
    BIT_CPU1_G,         LedGpio, addr => 0xB2, matrix => &MATRIX;

    MEM_R,              LedGpio, addr => 0x70, matrix => &MATRIX;
    MEM_G,              LedGpio, addr => 0x10, matrix => &MATRIX;

    RUN_R,              LedGpio, addr => 0x00, matrix => &MATRIX;
    RUN_G,              LedGpio, addr => 0x71, matrix => &MATRIX;
    UPDOG_G,            LedGpio, addr => 0xB1, matrix => &MATRIX;

    ///////////////////////////////////
    // Uncommitted LEDs
    UNC0_R,             LedGpio, addr => 0x95, matrix => &MATRIX;
    UNC0_G,             LedGpio, addr => 0x85, matrix => &MATRIX;
    UNC1_R,             LedGpio, addr => 0x94, matrix => &MATRIX;
    UNC1_G,             LedGpio, addr => 0x84, matrix => &MATRIX;
    UNC2_R,             LedGpio, addr => 0x92, matrix => &MATRIX;
    UNC2_G,             LedGpio, addr => 0x82, matrix => &MATRIX;
    UNC3_R,             LedGpio, addr => 0x93, matrix => &MATRIX;
    UNC3_G,             LedGpio, addr => 0x83, matrix => &MATRIX;
    UNC4_R,             LedGpio, addr => 0x90, matrix => &MATRIX;
    UNC4_G,             LedGpio, addr => 0x80, matrix => &MATRIX;
    UNC5_R,             LedGpio, addr => 0x91, matrix => &MATRIX;
    UNC5_G,             LedGpio, addr => 0x81, matrix => &MATRIX;
}
