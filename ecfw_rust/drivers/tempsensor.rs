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
use messages::Error;
use os::Mutex;

const TEMP_ADDR: u8 = 0u8;

pub struct TempSensor<'a> {
    twi: &'a Mutex<TwiDevice<'a>>
}

pub type TenthsDegC = i32;

impl<'a> TempSensor<'a> {
    pub const fn new(twi: &'a Mutex<TwiDevice<'a>>) -> TempSensor<'a> {
        TempSensor { twi: twi }
    }

    pub fn read(&self) -> Result<TenthsDegC,Error> {
        let mut buf = [0u8; 2];
        self.twi.lock().read(&[TEMP_ADDR], &mut buf)?;

        let raw = ((buf[0] as u32) << 8) | (buf[1] as u32);
        let right_aligned = raw >> 5;
        let masked = right_aligned & 0x7ff;
        let sign_extended =
            if (masked & 0x400) != 0 { masked | 0xfffff800 } else { masked };

        let eighths_degc = sign_extended as i32;

        Ok((10 * eighths_degc) / 8)
    }
}
