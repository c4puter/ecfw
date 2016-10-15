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

use hardware::gpio::*;
use hardware::ledmatrix::LedGpio;
use hardware::twi;
use hardware::twi::TwiDevice;
use hardware::gpio::Mode::*;

macro_rules! pin_table {
    (
        $( $name:ident, $kind:tt, $( $key:ident => $data:expr ),* );* ;
    ) => {
        pub static PIN_TABLE: &'static [&'static(Gpio + Sync)] = &[
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
static U101: TwiDevice = TwiDevice::new(twi::twi0, 0x21);

// PCF8575
const OUTPUTS_U901: u16 = 0xcfff;
static U901: TwiDevice = TwiDevice::new(twi::twi0, 0x20);

// AS1130
pub static U801: TwiDevice = TwiDevice::new(twi::twi0, 0x37);


pin_table!{

    ///////////////////////////////////
    // MCU GPIOs: FPGA interface, PCI, interrupts, etc

    BRIDGE_SUSP,        SamGpio, port => PIOC, pin => 31, mode => Output, default => false, invert => false;
    CARDEN,             SamGpio, port => PIOC, pin => 28, mode => Output, default => false, invert => true;
    CARD,               SamGpio, port => PIOA, pin => 20, mode => Input,  default => false, invert => true;
    CPU_SUSP,           SamGpio, port => PIOC, pin => 27, mode => Output, default => false, invert => false;
    FAN_PWM,            SamGpio, port => PIOA, pin => 22, mode => Output, default => false, invert => false;
    FAN_TACH,           SamGpio, port => PIOA, pin => 15, mode => Input,  default => false, invert => false;
    FPGA_CCLK,          SamGpio, port => PIOA, pin => 14, mode => Output, default => false, invert => false;
    FPGA_DATA,          SamGpio, port => PIOA, pin => 13, mode => Output, default => false, invert => false;
    FPGA_DONE0,         SamGpio, port => PIOA, pin =>  8, mode => Input,  default => false, invert => false;
    FPGA_DONE1,         SamGpio, port => PIOA, pin =>  7, mode => Input,  default => false, invert => false;
    FPGA_DONE2,         SamGpio, port => PIOA, pin =>  6, mode => Input,  default => false, invert => false;
    FPGA_INIT,          SamGpio, port => PIOA, pin =>  5, mode => Output, default => false, invert => false;
    PANELINT,           SamGpio, port => PIOC, pin => 17, mode => Input,  default => false, invert => true;
    PCIM66EN,           SamGpio, port => PIOC, pin => 24, mode => Input,  default => false, invert => false;
    PCIPME,             SamGpio, port => PIOC, pin => 23, mode => Input,  default => false, invert => true;
    PCIRST,             SamGpio, port => PIOC, pin => 22, mode => Output, default => false, invert => true;
    PROGRAM0,           SamGpio, port => PIOA, pin => 12, mode => Input,  default => false, invert => false;
    PROGRAM1,           SamGpio, port => PIOA, pin =>  9, mode => Input,  default => false, invert => false;
    PROGRAM2,           SamGpio, port => PIOA, pin => 10, mode => Input,  default => false, invert => false;
    PRSNT1_0,           SamGpio, port => PIOC, pin => 21, mode => Input,  default => false, invert => true;
    PRSNT1_1,           SamGpio, port => PIOC, pin => 16, mode => Input,  default => false, invert => true;
    PRSNT1_2,           SamGpio, port => PIOA, pin =>  2, mode => Input,  default => false, invert => true;
    PRSNT1_3,           SamGpio, port => PIOC, pin =>  9, mode => Input,  default => false, invert => true;
    PRSNT2_0,           SamGpio, port => PIOC, pin => 20, mode => Input,  default => false, invert => true;
    PRSNT2_1,           SamGpio, port => PIOC, pin => 19, mode => Input,  default => false, invert => true;
    PRSNT2_2,           SamGpio, port => PIOA, pin =>  1, mode => Input,  default => false, invert => true;
    PRSNT2_3,           SamGpio, port => PIOC, pin => 10, mode => Input,  default => false, invert => true;
    REQ,                SamGpio, port => PIOA, pin => 16, mode => Input,  default => false, invert => false;
    RTCINT,             SamGpio, port => PIOC, pin => 25, mode => Input,  default => false, invert => true;
    SDRAM_EVENT,        SamGpio, port => PIOC, pin => 18, mode => Input,  default => false, invert => true;
    SDRAM_RST,          SamGpio, port => PIOB, pin => 13, mode => Output, default => false, invert => true;
    USB_VBSENSE,        SamGpio, port => PIOA, pin =>  0, mode => Input,  default => false, invert => false;
    VREFEN,             SamGpio, port => PIOB, pin =>  0, mode => Output, default => false, invert => false;

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
    EN_1V2,             PcfGpio, dev => &U901, pin =>  2, outputs => OUTPUTS_U901, default => false, invert => false;
    EN_1V5,             PcfGpio, dev => &U901, pin =>  3, outputs => OUTPUTS_U901, default => false, invert => false;
    EN_5V_PCI_B,        PcfGpio, dev => &U901, pin => 15, outputs => OUTPUTS_U901, default => false, invert => true;
    EN_P12V_PCI,        PcfGpio, dev => &U901, pin => 16, outputs => OUTPUTS_U901, default => false, invert => false;
    EN_P3V3_S0B,        PcfGpio, dev => &U901, pin => 12, outputs => OUTPUTS_U901, default => false, invert => true;
    EN_SAFETY,          PcfGpio, dev => &U901, pin =>  6, outputs => OUTPUTS_U901, default => true,  invert => false;
    EN_V75,             PcfGpio, dev => &U901, pin =>  0, outputs => OUTPUTS_U901, default => false, invert => false;
    EN_V75REF,          PcfGpio, dev => &U901, pin =>  1, outputs => OUTPUTS_U901, default => false, invert => false;

    ///////////////////////////////////
    // Power LEDs

    P12V_PCI_R,         LedGpio, addr => 0x75;
    P12V_PCI_G,         LedGpio, addr => 0x44;
    P5V_PCI_A_R,        LedGpio, addr => 0x54;
    P5V_PCI_A_G,        LedGpio, addr => 0x74;

    P5V_PCI_B_R,        LedGpio, addr => 0x52;
    P5V_PCI_B_G,        LedGpio, addr => 0x42;

    P3V3_PCI_A_R,       LedGpio, addr => 0x53;
    P3V3_PCI_A_G,       LedGpio, addr => 0x43;

    P3V3_PCI_B_R,       LedGpio, addr => 0x50;
    P3V3_PCI_B_G,       LedGpio, addr => 0x40;

    N12V_PCI_R,         LedGpio, addr => 0x51;
    N12V_PCI_G,         LedGpio, addr => 0x41;

    P3V3_STBY_R,        LedGpio, addr => 0x25;
    P3V3_STBY_G,        LedGpio, addr => 0x34;

    P3V3_AUX_R,         LedGpio, addr => 0x23;
    P3V3_AUX_G,         LedGpio, addr => 0x33;

    P3V3_LOGIC_R,       LedGpio, addr => 0x72;
    P3V3_LOGIC_G,       LedGpio, addr => 0x32;

    P1V5_LOGIC_R,       LedGpio, addr => 0x22;
    P1V5_LOGIC_G,       LedGpio, addr => 0x73;

    P1V2_LOGIC_R,       LedGpio, addr => 0x20;
    P1V2_LOGIC_G,       LedGpio, addr => 0x30;

    PV75_TERM_R,        LedGpio, addr => 0x21;
    PV75_TERM_G,        LedGpio, addr => 0x31;

    ///////////////////////////////////
    // Boot sequence LEDs
    ECFW_R,             LedGpio, addr => 0x04;
    ECFW_G,             LedGpio, addr => 0x14;

    POWER_R,            LedGpio, addr => 0x03;
    POWER_G,            LedGpio, addr => 0x31;

    CARD_R,             LedGpio, addr => 0x01;
    CARD_G,             LedGpio, addr => 0x11;

    BIT_R,              LedGpio, addr => 0x02;
    BIT_BRIDGE_G,       LedGpio, addr => 0x12;
    BIT_CPU0_G,         LedGpio, addr => 0xA2;
    BIT_CPU1_G,         LedGpio, addr => 0xB2;

    MEM_R,              LedGpio, addr => 0x70;
    MEM_G,              LedGpio, addr => 0x10;

    RUN_R,              LedGpio, addr => 0x00;
    RUN_G,              LedGpio, addr => 0x71;
    UPDOG_G,            LedGpio, addr => 0xB1;

    ///////////////////////////////////
    // Uncommitted LEDs
    UNC0_R,             LedGpio, addr => 0x95;
    UNC0_G,             LedGpio, addr => 0x85;
    UNC1_R,             LedGpio, addr => 0x94;
    UNC1_G,             LedGpio, addr => 0x84;
    UNC2_R,             LedGpio, addr => 0x92;
    UNC2_G,             LedGpio, addr => 0x82;
    UNC3_R,             LedGpio, addr => 0x93;
    UNC3_G,             LedGpio, addr => 0x83;
    UNC4_R,             LedGpio, addr => 0x90;
    UNC4_G,             LedGpio, addr => 0x80;
    UNC5_R,             LedGpio, addr => 0x91;
    UNC5_G,             LedGpio, addr => 0x81;
}
