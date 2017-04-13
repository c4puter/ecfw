/*
 * The MIT License (MIT)
 * Copyright (c) 2017 Chris Pavlina
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

use hardware::twi::TwiDevice;
use main::twi_devices;
use main::messages::Error;
use rustsys::mutex::Mutex;

const TEMP_ADDR: u8 = 0u8;

pub static SENSOR_LOGIC: TempSensor = TempSensor::new(&twi_devices::LM75B_LOGIC);
pub static SENSOR_AMBIENT: TempSensor = TempSensor::new(&twi_devices::LM75B_AMBIENT);

pub struct TempSensor {
    twi: &'static Mutex<TwiDevice>
}

pub type TenthsDegC = i32;

impl TempSensor {
    pub const fn new(twi: &'static Mutex<TwiDevice>) -> TempSensor {
        TempSensor { twi: twi }
    }

    pub fn read(&self) -> Result<TenthsDegC,Error> {
        let mut buf = [0u8; 2];
        try!(self.twi.lock().read(&[TEMP_ADDR], &mut buf));

        let raw = ((buf[0] as u32) << 8) | (buf[1] as u32);
        let right_aligned = raw >> 5;
        let masked = right_aligned & 0x7ff;
        let sign_extended =
            if (masked & 0x400) != 0 { masked | 0xfffff800 } else { masked };

        let eighths_degc = sign_extended as i32;

        Ok((10 * eighths_degc) / 8)
    }
}
