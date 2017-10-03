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

use drivers::twi::TwiDevice;
use messages::*;
use os::Mutex;
use os;
use rustsys::ec_io;
use bindgen_mcu;

pub struct ClockSynth<'a> {
    twi: &'a Mutex<TwiDevice<'a>>,
    xtal: u32,
}

impl<'a> ClockSynth<'a> {
    /// Create a ClockSynth instance for a CDCE913.
    /// twi: TwiDevice for this CDCE913.
    /// xtal: Frequency of the crystal (or other clock input) in Hz.
    pub const fn new(twi: &'a Mutex<TwiDevice<'a>>, xtal: u32) -> ClockSynth<'a>
    {
        ClockSynth { twi: twi, xtal: xtal }
    }

    /// Set the Y1 output divider, from 1 to 1023
    pub fn y1div(&self, div: u32) -> StdResult
    {
        if div < 1 || div > 1023 {
            Err(ERR_NRANGE)
        } else {
            let reg02 = 0xb4 | ((div & 0x300) >> 8) as u8;
            let reg03 = (div & 0xff) as u8;
            self.twi.lock().write(&[0x02], &[2, reg02, reg03])
        }
    }

    /// Set the Y2 output divider, from 1 to 127
    pub fn y2div(&self, div: u32) -> StdResult
    {
        if div < 1 || div > 127 {
            Err(ERR_NRANGE)
        } else {
            let mut reg16 = [0u8; 1];
            let mut twilock = self.twi.lock();
            twilock.read(&[0x16 | 0x80], &mut reg16)?;

            reg16[0] &= 0x7f;
            reg16[0] |= div as u8;

            twilock.write(&[0x16], &[1, reg16[0]])
        }
    }

    /// Set the Y3 output divider, from 1 to 127
    pub fn y3div(&self, div: u32) -> StdResult
    {
        if div < 1 || div > 127 {
            Err(ERR_NRANGE)
        } else {
            let reg17 = div as u8;
            self.twi.lock().write(&[0x17], &[1, reg17])
        }
    }

    /// Set the load capacitance from 0 to 20pF
    pub fn loadcap(&self, pf: u32) -> StdResult
    {
        if pf > 20 {
            Err(ERR_NRANGE)
        } else {
            let reg05 = (pf << 3) as u8;
            self.twi.lock().write(&[0x05], &[1, reg05])
        }
    }

    /// Enable or disable the PLL
    pub fn usepll(&self, v: bool) -> StdResult
    {
        let mut reg14 = [0u8; 1];
        let mut twilock = self.twi.lock();
        twilock.read(&[0x14 | 0x80], &mut reg14)?;

        if v {
            reg14[0] &= !0x80;
        } else {
            reg14[0] |= 0x80;
        }

        twilock.write(&[0x14], &[1, reg14[0]])?;
        Ok(())
    }

    // 20 MHz * 75 / 8  =  187.5 MHz
    // 187.5 MHz / 3 = 62.5 MHz  (bridge clock)
    // 187.5 MHz / 2 = 93.75 MHz (CPU clock)

    /// Set the PLL ratio. 1 <= num <= 4095, 1 <= den <= 511, and
    /// 80MHz <= freq <= 230MHz.
    pub fn ratio(&self, num: u32, den: u32) -> StdResult
    {
        if num < 1 || num > 4095 || den < 1 || den > 511 {
            return Err(ERR_NRANGE);
        }

        let freq = (self.xtal as u64) * (num as u64) / den as u64;

        if freq < 80000000 || freq > 230000000 {
            return Err(ERR_PLL_RANGE);
        }

        let n = num;
        let m = den;
        // log2(16 * x) - 4 == log2(x)
        // This ensures sub-integer values are accounted for because I'm too
        // lazy to check whether they matter
        let p = 4u32.saturating_sub(log2(16 * n / m).saturating_sub(4));
        let np = n * (1 << p);
        let q = np / m;
        let r = np - m * q;

        let frange = match freq {
            0 ... 124999999 => 0,
            125000000 ... 149999999 => 1,
            150000000 ... 174999999 => 2,
            _ => 3 };

        debug!(DEBUG_CLOCK,
               "configuring PLL with P={}, N={}. N'={}, Q={}, R={}",
                p, n, np, q, r);

        let reg18 = ((n & 0xff0) >> 4) as u8;
        let reg19 = (((n & 0x00f) << 4) | ((r & 0x1e0) >> 5)) as u8;
        let reg1a = (((r & 0x1f) << 3) | ((q & 0x38) >> 3)) as u8;
        let reg1b = (((q & 0x07) << 5) | ((p & 0x03) << 2) | frange) as u8;

        let mut twilock = self.twi.lock();
        twilock.write(&[0x18], &[1, reg18])?;
        twilock.write(&[0x19], &[1, reg19])?;
        twilock.write(&[0x1a], &[1, reg1a])?;
        twilock.write(&[0x1b], &[1, reg1b])?;
        Ok(())
    }

    /// Switch to the input from the clock synthesizer as this MCU's clock
    /// source. This is VERY unsafe: the resulting clock frequency must 1)
    /// be valid within the microcontroller's requirements (subject to the
    /// external crystal frequency, the values passed to the synthesizer,
    /// and the internal configuration in conf_clock.h), and 2) be equal to
    /// BOARD_MCK if you don't want serial baud rates to go wonky.
    ///
    /// This PANICs if some of its checks fail, and doesn't check everything.
    ///
    /// Beware.
    pub unsafe fn enable_mck(&self)
    {
        let mut twilock = self.twi.lock();
        debug!(DEBUG_CLOCK, "switching to external clock");
        ec_io::flush_output();
        os::freertos::suspend_all();
        let mut buf = [0u8];
        if let Err(_) = twilock.read(&[0x80], &mut buf) {
            panic!("Tried to enable external clock when main I2C is down");
        }

        bindgen_mcu::mcu_use_external_clock(true);
        os::freertos::resume_all();
    }

    /// Switch back to the internal clock. This is safer, but you're still
    /// messing with clock sources.
    pub unsafe fn disable_mck(&self)
    {
        debug!(DEBUG_CLOCK, "switching to internal clock");
        ec_io::flush_output();
        os::freertos::suspend_all();
        bindgen_mcu::mcu_use_external_clock(false);
        os::freertos::resume_all();
    }
}

fn log2(n: u32) -> u32
{
    31u32 - n.leading_zeros()
}
