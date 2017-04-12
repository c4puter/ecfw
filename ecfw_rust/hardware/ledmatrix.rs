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

use rustsys::freertos;
use rustsys::mutex::{Mutex, MutexLock};
use rustsys::rwlock::RwLock;
use hardware::{gpio, twi};
use main::messages::*;
use main::twi_devices;

/// LED current in mA. Maximum is 30mA.
pub const LED_CURRENT: f32 = 15.0;

/// Brightness (of 255) when in standby
pub const STANDBY_BRIGHTNESS: u8 = 40;

/// Brightness (of 255) when runnint
pub const FULL_BRIGHTNESS: u8 = 255;

/// LED current in AS1130 units
pub const LED_CURRENT_AS1130: u8 = (LED_CURRENT / 0.11765) as u8;

pub static MATRIX: RwLock<LedMatrix> =
    RwLock::new(LedMatrix::new(&twi_devices::U801 ));

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
    twi: &'static Mutex<twi::TwiDevice>,
    buffer: [u8; 24],
}

impl LedMatrix
{
    pub const fn new(twi: &'static Mutex<twi::TwiDevice>) -> LedMatrix
    {
        LedMatrix {
            twi: twi,
            buffer: [0u8; 24],
        }
    }

    pub fn init(&mut self) -> StdResult
    {
        freertos::delay(6);

        let mut dev = self.twi.lock();

        // Define RAM configuration, bit mem_conf in config register
        //  - On/Off frames
        //  - Blink/PWM sets
        //  - Dot correction
        //  0x01: RAM configuration 1
        try!(self.switch_bank(&mut dev, RegBank::ControlReg));
        try!(dev.write(&[CtrlReg::Config as u8], &[0x01]));

        // Define control register
        //  - Current source
        //  - Display options
        //  - Display picture/play movie
        try!(dev.write(&[CtrlReg::CurrentSource as u8],
                       &[LedMatrix::current_val(STANDBY_BRIGHTNESS)]));
        try!(dev.write(&[CtrlReg::DisplayOpt as u8], &[0xfb])); // Scan all segments
        try!(dev.write(&[CtrlReg::Movie as u8], &[0x00])); // No movie
        try!(dev.write(&[CtrlReg::Picture as u8], &[0x40])); // Display picture, frame 0
        // Set #shdn bit to 1 for normal operation
        try!(dev.write(&[CtrlReg::ShutdownOpenShort as u8], &[0x03])); // No init, no shutdown
        freertos::delay(1);

        // Initialize display data
        try!(self.switch_bank(&mut dev, RegBank::BlinkPwm0));
        for seg in 0x00..0x0c {
            // Blink bits
            try!(dev.write(&[seg*2], &[0, 0]));
        }
        for addr in 0x18..0x9c {
            // PWM value
            try!(dev.write(&[addr], &[0x80]));
        }

        self.buffer_all(true);
        try!(self.flush_with_lock(&mut dev));
        Ok(())
    }

    fn switch_bank(&mut self, dev: &mut MutexLock<twi::TwiDevice>, bank: RegBank) -> StdResult
    {
        dev.write(&[REG_BANK_SELECT], &[bank as u8])
    }

    fn flush_with_lock(&mut self, mut dev: &mut MutexLock<twi::TwiDevice>) -> StdResult
    {
        try!(self.switch_bank(&mut dev, RegBank::Frame0));

        for seg in 0x00usize..0x0cusize {
            try!(dev.write(&[(seg*2) as u8],
                  &[self.buffer[seg*2], self.buffer[seg*2 + 1]]));
        }
        Ok(())
    }

    pub fn flush(&mut self) -> StdResult
    {
        let mut twi = self.twi.lock();
        self.flush_with_lock(&mut twi)
    }

    pub fn buffer_all(&mut self, val: bool)
    {
        let regval = match val {
            true  => [0xff, 0x07],
            false => [0x00, 0x00],
        };

        {
            for seg in 0x00..0x0c {
                self.buffer[seg*2] = regval[0];
                self.buffer[seg*2 + 1] = regval[1];
            }
        }
    }

    pub fn buffer_led(&mut self, led: u8, val: bool)
    {
        let segment = (led & 0xf0) >> 4;
        let addr = (2 * segment) as usize;
        let bit = 1 << (led & 0x0f);

        let buffer = &mut self.buffer[addr..addr+2];

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
    }

    pub fn set_led(&mut self, led: u8, val: bool) -> StdResult
    {
        let segment = (led & 0xf0) >> 4;
        let addr = (2 * segment) as usize;

        self.buffer_led(led, val);

        let mut dev = self.twi.lock();
        try!(self.switch_bank(&mut dev, RegBank::Frame0));
        let buffer = &mut self.buffer[addr..addr+2];
        try!(dev.write(&[addr as u8], buffer));

        Ok(())
    }

    pub fn get_led(&self, led: u8) -> bool
    {
        let segment = (led & 0xf0) >> 4;
        let addr = (2 * segment) as usize;
        let bit = 1 << (led & 0x0f);
        let buffer = &self.buffer[addr..addr+2];

        let register = (buffer[0] as u16) | ((buffer[1] as u16) << 8);

        register & bit != 0
    }

    /// Set the LED brightness in 1/256 of LED_CURRENT.
    pub fn set_brightness(&mut self, brightness: u8) -> StdResult
    {
        let mut dev = self.twi.lock();
        try!(self.switch_bank(&mut dev, RegBank::ControlReg));
        try!(dev.write(&[CtrlReg::CurrentSource as u8],
                       &[LedMatrix::current_val(brightness)]));
        Ok(())
    }

    /// Get the raw current value for a brightness
    fn current_val(brightness: u8) -> u8
    {
        ((brightness as u32 * LED_CURRENT_AS1130 as u32) / 256) as u8
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
        MATRIX.write().set_led(self.addr, v).unwrap();
    }

    fn get(&self) -> bool {
        MATRIX.read().get_led(self.addr)
    }

    fn name(&self) -> &'static str {
        self.name
    }
}
