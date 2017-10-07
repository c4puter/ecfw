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

use os;
use drivers::{gpio, twi};
use messages::*;

/// LED current in mA. Maximum is 30mA.
pub const LED_CURRENT: f32 = 15.0;

/// Brightness (of 255) when in standby
pub const STANDBY_BRIGHTNESS: u8 = 40;

/// Brightness (of 255) when runnint
pub const FULL_BRIGHTNESS: u8 = 255;

/// LED current in AS1130 units
pub const LED_CURRENT_AS1130: u8 = (LED_CURRENT / 0.11765) as u8;

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

pub struct LedMatrix<'a>
{
    twi: &'a os::Mutex<twi::TwiDevice<'a>>,
    buffer: [u8; 24],
    blinkbuf: [u8; 24],
}

impl<'a> LedMatrix<'a>
{
    pub const fn new(twi: &'a os::Mutex<twi::TwiDevice<'a>>) -> LedMatrix<'a>
    {
        LedMatrix {
            twi: twi,
            buffer: [0u8; 24],
            blinkbuf: [0u8; 24],
        }
    }

    pub fn init(&mut self) -> StdResult
    {
        os::delay(6);

        let mut dev = self.twi.lock();

        // Define RAM configuration, bit mem_conf in config register
        //  - On/Off frames
        //  - Blink/PWM sets
        //  - Dot correction
        //  0x01: RAM configuration 1
        self.switch_bank(&mut dev, RegBank::ControlReg)?;
        dev.write(&[CtrlReg::Config as u8], &[0x01])?;

        // Define control register
        //  - Current source
        //  - Display options
        //  - Display picture/play movie
        dev.write(&[CtrlReg::CurrentSource as u8],
                  &[LedMatrix::current_val(STANDBY_BRIGHTNESS)])?;
        dev.write(&[CtrlReg::DisplayOpt as u8], &[0xfb])?; // Scan all segments
        dev.write(&[CtrlReg::Movie as u8], &[0x00])?; // No movie
        dev.write(&[CtrlReg::Picture as u8], &[0x40])?; // Display picture, frame 0
        // Set #shdn bit to 1 for normal operation
        dev.write(&[CtrlReg::ShutdownOpenShort as u8], &[0x03])?; // No init, no shutdown
        os::delay(1);

        // Initialize display data
        self.switch_bank(&mut dev, RegBank::BlinkPwm0)?;
        for seg in 0x00..0x0c {
            // Blink bits
            dev.write(&[seg*2], &[0, 0])?;
        }
        for addr in 0x18..0x9c {
            // PWM value
            dev.write(&[addr], &[0x80])?;
        }

        self.buffer_all(true);
        self.flush_with_lock(&mut dev)?;
        Ok(())
    }

    fn switch_bank(&mut self, dev: &mut os::MutexLock<twi::TwiDevice>, bank: RegBank) -> StdResult
    {
        dev.write(&[REG_BANK_SELECT], &[bank as u8])
    }

    fn flush_with_lock(&mut self, mut dev: &mut os::MutexLock<twi::TwiDevice>) -> StdResult
    {
        self.switch_bank(&mut dev, RegBank::Frame0)?;

        for seg in 0x00usize..0x0cusize {
            dev.write(&[(seg*2) as u8],
                  &[self.buffer[seg*2], self.buffer[seg*2 + 1]])?;
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

    pub fn buffer_led(&mut self, led: u8, val: bool, blink: bool)
    {
        let segment = (led & 0xf0) >> 4;
        let addr = (2 * segment) as usize;

        let buffer = &mut self.buffer[addr..addr+2];
        let blinkbuf = &mut self.blinkbuf[addr..addr+2];

        write_bit(buffer, (led & 0x0f) as _, val);
        write_bit(blinkbuf, (led & 0x0f) as _, blink);

        buffer[1] &= 0x07;
        blinkbuf[1] &= 0x07;
    }

    pub fn set_led(&mut self, led: u8, val: bool, blink: bool) -> StdResult
    {
        let segment = (led & 0xf0) >> 4;
        let addr = (2 * segment) as usize;

        self.buffer_led(led, val, blink);

        let mut dev = self.twi.lock();
        self.switch_bank(&mut dev, RegBank::Frame0)?;
        dev.write(&[addr as u8], &mut self.buffer[addr..addr+2])?;

        self.switch_bank(&mut dev, RegBank::BlinkPwm0)?;
        dev.write(&[addr as u8], &mut self.blinkbuf[addr..addr+2])?;

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
        self.switch_bank(&mut dev, RegBank::ControlReg)?;
        dev.write(&[CtrlReg::CurrentSource as u8],
                       &[LedMatrix::current_val(brightness)])?;
        Ok(())
    }

    /// Set the LED brightness to "full"
    pub fn set_full_brightness(&mut self) -> StdResult
    {
        self.set_brightness(FULL_BRIGHTNESS)
    }

    /// Set the LED brightness to "standby"
    pub fn set_standby_brightness(&mut self) -> StdResult
    {
        self.set_brightness(STANDBY_BRIGHTNESS)
    }

    /// Get the raw current value for a brightness
    fn current_val(brightness: u8) -> u8
    {
        ((brightness as u32 * LED_CURRENT_AS1130 as u32) / 256) as u8
    }
}

fn write_bit(buffer: &mut [u8], bit: usize, val: bool)
{
    if val {
        buffer[bit / 8] |= 1 << (bit % 8);
    } else {
        buffer[bit / 8] &= !(1 << (bit % 8));
    }
}

pub struct LedGpio<'a>
{
    pub addr: u8,
    pub matrix: &'a os::RwLock<LedMatrix<'a>>,
    pub name: &'static str,
}

impl<'a> LedGpio<'a>
{
    pub fn set_blink(&self) {
        self.matrix.write().set_led(self.addr, true, true).unwrap();
    }
}

impl<'a> gpio::Gpio for LedGpio<'a>
{
    fn init(&self) {}

    fn set(&self, v: bool) {
        self.matrix.write().set_led(self.addr, v, false).unwrap();
    }

    fn get(&self) -> bool {
        self.matrix.read().get_led(self.addr)
    }

    fn name(&self) -> &'static str {
        self.name
    }
}
