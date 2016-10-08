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
extern crate bindgen_mcu;
extern crate twi;

pub trait Gpio {
    fn init(&self);
    fn set(&self, v: bool);
    fn get(&self) -> bool;
    fn name(&self) -> &'static str;
}

// These are 32x the values defined in ioport_pio.h, as IOPORT_CREATE_PIN
// multiplies them by 32 and we can't do that "constexpr".
pub type Ioport = u32;
pub const PIOA: Ioport = 0 * 32;
pub const PIOB: Ioport = 1 * 32;
pub const PIOC: Ioport = 2 * 32;
pub const PIOD: Ioport = 3 * 32;
pub const PIOE: Ioport = 4 * 32;
pub const PIOF: Ioport = 5 * 32;

#[derive(PartialEq)]
pub enum Mode { Input, Output }

pub struct SamGpio {
    pub port: Ioport,
    pub pin: u32,
    pub mode: Mode,
    pub default: bool,
    pub invert: bool,
    pub name: &'static str,
}

impl Gpio for SamGpio {
    fn init(&self) {
        unsafe {
            bindgen_mcu::mcu_init_pin(
                self.port + self.pin,
                (self.mode == Mode::Output) as u8,
                self.default as u8);
        }
    }

    fn set(&self, v: bool) {
        unsafe {
            let inv_v = if self.invert {!v} else {v};
            bindgen_mcu::mcu_set_pin_level(self.port + self.pin, inv_v as u8);
        }
    }

    fn get(&self) -> bool {
        unsafe {
            let v = bindgen_mcu::mcu_get_pin_level(self.port + self.pin) != 0;
            if self.invert {!v} else {v}
        }
    }

    fn name(&self) -> &'static str { self.name }
}

pub struct PcfGpio {
    pub dev: &'static twi::TwiDevice,
    pub pin: u8,
    pub default: bool,
    pub invert: bool,

    /// WARNING: The PCF8575 provides no way to read back programmed values,
    /// only actual values for all pins. This field must therefore contain a
    /// bitmask, with a 1 for every pin on the given device that is to be an
    /// output.
    ///
    /// MSB.........................................LSB
    /// 7  6  5  4  3  2  1  0  17 16 15 14 13 12 11 10
    pub outputs: u16,
    pub name: &'static str,
}

impl Gpio for PcfGpio {
    fn init(&self) {
        self.set(self.default);
    }

    fn set(&self, v: bool) {
        // Write to the chip:
        // MSB................LSB  MSB.................LSB
        // 7  6  5  4  3  2  1  0  17 16 15 14 13 12 11 10

        let inv_v = if self.invert {!v} else {v};

        let mut data = [0 as u8; 2];
        let pinbit =
            if self.pin <= 7                            { 1 << (self.pin + 8) }
            else if self.pin <= 17 && self.pin >= 10    { 1 << (self.pin - 10) }
            else { panic!("invalid pin number {}", self.pin); };

        let _lock = self.dev.lock();
        self.dev.read(&[], &mut data).unwrap();

        let mut data_u16 = (data[1] as u16) | ((data[0] as u16) << 8);

        // The PCF8575 doesn't have output modes. Output has only a weak driver,
        // with input and output-with-pullup being the same mode. Mask off any
        // inputs as output-with-pullup:
        data_u16 = (self.outputs & data_u16) | !self.outputs;
        if inv_v {
            data_u16 |= pinbit;
        } else {
            data_u16 &= !pinbit;
        }

        data[0] = ((data_u16 >> 8) & 0xff) as u8;
        data[1] = ((data_u16)      & 0xff) as u8;

        self.dev.write(&[], &data).unwrap();
    }

    fn get(&self) -> bool {
        let mut data = [0 as u8; 2];
        let pinbit =
            if self.pin <= 7                            { 1 << (self.pin + 8) }
            else if self.pin <= 17 && self.pin >= 10    { 1 << (self.pin - 10) }
            else { panic!("invalid pin number {}", self.pin); };

        let _lock = self.dev.lock();
        self.dev.read(&[], &mut data).unwrap();

        let data_u16 = (data[1] as u16) | ((data[0] as u16) << 8);

        let v = data_u16 & pinbit != 0;
        if self.invert { !v } else { v }
    }

    fn name(&self) -> &'static str { self.name }
}
