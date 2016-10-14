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

use rustsys::freertos;
use hardware::{gpio, twi};
use hardware::twi::TwiResult;

static mut MATRIX: LedMatrix = LedMatrix{twi: None};
pub fn matrix() -> &'static LedMatrix { unsafe{&MATRIX} }
pub unsafe fn matrix_init(twi: &'static twi::TwiDevice) -> Result<(),TwiResult> { MATRIX.init(twi) }

// AS1130 register banks
#[allow(dead_code)]
#[repr(u8)]
enum RegBank
{
    Nop = 0x00,
    Frame0 = 0x01,
    // Frame1 through Frame35 are 0x02 through 0x24. Not going to bother
    // defining them as we'll only ever use Frame0
    BlinkPwm0 = 0x40,
    // As above for BlinkPwm1 through BlinkPwm5
    DotCorrection = 0x80,
    ControlReg = 0xc0,
}

// Bank select address
const REG_BANK_SELECT: u8 = 0xfd;

#[allow(dead_code)]
#[repr(u8)]
enum CtrlReg
{
    Picture = 0x00,
    Movie = 0x01,
    MovieMode = 0x02,
    FrameTime = 0x03,
    DisplayOpt = 0x04,
    CurrentSource = 0x05,
    Config = 0x06,
    InterruptMask = 0x07,
    InterruptFrame = 0x08,
    ShutdownOpenShort = 0x09,
    I2CMonitor = 0x0a,
    ClkSync = 0x0b,
    InterruptStatus = 0x0e,
    Status = 0x0f,
}

pub struct LedMatrix
{
    twi: Option<&'static twi::TwiDevice>,
}

impl LedMatrix
{
    pub fn init(&mut self, twi: &'static twi::TwiDevice) -> Result<(),TwiResult>
    {
        self.twi = Some(twi);
        freertos::delay(6);

        let _lock = twi.lock();

        // Define RAM configuration, bit mem_conf in config register
        //  - On/Off frames
        //  - Blink/PWM sets
        //  - Dot correction
        //  0x01: RAM configuration 1
        try!(self.switch_bank(RegBank::ControlReg));
        try!(twi.write(&[CtrlReg::Config as u8], &[0x01]));

        // Define control register
        //  - Current source
        //  - Display options
        //  - Display picture/play movie
        try!(twi.write(&[CtrlReg::CurrentSource as u8], &[170])); // 20mA / 117.65uA
        try!(twi.write(&[CtrlReg::DisplayOpt as u8], &[0xfb])); // Scan all segments
        try!(twi.write(&[CtrlReg::Movie as u8], &[0x00])); // No movie
        try!(twi.write(&[CtrlReg::Picture as u8], &[0x40])); // Display picture, frame 0
        // Set #shdn bit to 1 for normal operation
        try!(twi.write(&[CtrlReg::ShutdownOpenShort as u8], &[0x03])); // No init, no shutdown
        freertos::delay(1);

        // Initialize display data
        try!(self.switch_bank(RegBank::BlinkPwm0));
        for seg in 0x00..0x0c {
            // Blink bits
            try!(twi.write(&[seg*2], &[0, 0]));
        }
        for addr in 0x18..0x9c {
            // PWM value
            try!(twi.write(&[addr], &[0x80]));
        }

        try!(self.set_all(true));
        Ok(())
    }

    fn switch_bank(&self, bank: RegBank) -> Result<(),TwiResult>
    {
        self.twi.unwrap().write(&[REG_BANK_SELECT], &[bank as u8])
    }

    pub fn set_all(&self, val: bool) -> Result<(), TwiResult>
    {
        let ref twi = self.twi.unwrap();

        // Ensure PWM selector bits are zero
        let regval = match val {
            true  => [0xff, 0x07],
            false => [0x00, 0x00],
        };
        try!(self.switch_bank(RegBank::Frame0));
        for seg in 0x00..0x0c {
            try!(twi.write(&[seg*2], &regval));
        }
        Ok(())
    }

    pub fn set_led(&self, led: u8, val: bool) -> Result<(), TwiResult>
    {
        let segment = (led & 0xf0) >> 4;
        let addr = 2 * segment;
        let bit = 1 << (led & 0x0f);

        let _lock = self.twi.unwrap().lock();
        try!(self.switch_bank(RegBank::Frame0));
        let mut buffer = [0 as u8; 2];

        try!(self.twi.unwrap().read(&[addr], &mut buffer));

        let mut register: u16 = (buffer[0] as u16) | ((buffer[1] as u16) << 8);

        if val {
            register |= bit;
        } else {
            register &= !bit;
        }

        // Ensure PWM selector bits are zero
        register &= 0x07ff;

        buffer[0] = (register & 0xff) as u8;
        buffer[1] = ((register & 0xff00) >> 8) as u8;

        try!(self.twi.unwrap().write(&[addr], &buffer));

        Ok(())
    }

    pub fn get_led(&self, led: u8) -> Result<bool, TwiResult>
    {
        let segment = (led & 0xf0) >> 4;
        let addr = 2 * segment;
        let bit = 1 << (led & 0x0f);
        let mut buffer = [0 as u8; 2];

        let _lock = self.twi.unwrap().lock();

        try!(self.switch_bank(RegBank::Frame0));
        try!(self.twi.unwrap().read(&[addr], &mut buffer));

        let register = (buffer[0] as u16) | ((buffer[1] as u16) << 8);

        Ok(register & bit != 0)
    }
}

pub struct LedGpio
{
    pub addr: u8,
    pub name: &'static str,
}

impl gpio::Gpio for LedGpio
{
    fn init(&self) {}

    fn set(&self, v: bool) {
        matrix().set_led(self.addr, v).unwrap();
    }

    fn get(&self) -> bool {
        matrix().get_led(self.addr).unwrap()
    }

    fn name(&self) -> &'static str {
        self.name
    }
}
