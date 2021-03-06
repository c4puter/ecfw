// c4puter embedded controller firmware
// Copyright (C) 2017 Chris Pavlina
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//

//! Drivers for on-chip and off-chip GPIOs.

use bindgen_mcu;

use drivers::i2c;
use os::Mutex;

pub trait Gpio {
    fn init(&self);
    fn set(&self, v: bool);
    fn get(&self) -> bool;
    fn name(&self) -> &'static str;
}

// These are 32x the values defined in ioport_pio.h, as IOPORT_CREATE_PIN
// multiplies them by 32 and we can't do that "constexpr".
pub type Ioport = u32;
#[allow(dead_code)] pub const PIOA: Ioport = 0 * 32;
#[allow(dead_code)] pub const PIOB: Ioport = 1 * 32;
#[allow(dead_code)] pub const PIOC: Ioport = 2 * 32;
#[allow(dead_code)] pub const PIOD: Ioport = 3 * 32;
#[allow(dead_code)] pub const PIOE: Ioport = 4 * 32;
#[allow(dead_code)] pub const PIOF: Ioport = 5 * 32;

// Mode values are chosen for easy translation to the ones ASF uses.
// bit 31 on = peripheral mode
// bit 30 on = output
// value & 0xffff = ASF ioport mode
#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
#[repr(u32)]
pub enum Mode {
    Input   = 0x00000000,
    Output  = 0x40000000,
    Pullup  = 0x00000008,
    Pulldn  = 0x00000010,
    OpnDrn  = 0x40000020,
    ODPull  = 0x40000028,
    PerA    = 0x80000000,
    PerB    = 0x80000001,
    PerC    = 0x80000002,
    PerD    = 0x80000003,
}

pub struct SamGpio {
    pub port: Ioport,
    pub pin: u32,
    pub mode: Mode,
    pub default: bool,
    pub invert: bool,
    pub name: &'static str,
}

impl Gpio for SamGpio {
    fn init(&self)
    {
        unsafe {
            bindgen_mcu::mcu_init_pin(
                self.port + self.pin,
                self.mode as u32,
                self.default ^ self.invert,
            );
        }
    }

    fn set(&self, v: bool)
    {
        unsafe {
            let inv_v = v ^ self.invert;
            bindgen_mcu::mcu_set_pin_level(self.port + self.pin, inv_v);
        }
    }

    fn get(&self) -> bool
    {
        unsafe {
            let v = bindgen_mcu::mcu_get_pin_level(self.port + self.pin);
            v ^ self.invert
        }
    }

    fn name(&self) -> &'static str
    {
        self.name
    }
}

unsafe impl Sync for SamGpio {}

pub struct PcfGpio<'a> {
    pub dev: &'a Mutex<i2c::I2CDevice<'a>>,
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

impl<'a> Gpio for PcfGpio<'a> {
    fn init(&self)
    {
        self.set(self.default);
    }

    fn set(&self, v: bool)
    {
        // Write to the chip:
        // MSB................LSB  MSB.................LSB
        // 7  6  5  4  3  2  1  0  17 16 15 14 13 12 11 10

        let inv_v = v ^ self.invert;

        let mut data = [0 as u8; 2];
        let pinbit = if self.pin <= 7 {
            1 << (self.pin + 8)
        } else if self.pin <= 17 && self.pin >= 10 {
            1 << (self.pin - 10)
        } else {
            panic!("invalid pin number {}", self.pin);
        };

        let mut dev = self.dev.lock();
        dev.read(&[], &mut data).unwrap();

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
        data[1] = ((data_u16) & 0xff) as u8;

        dev.write(&[], &data).unwrap();
    }

    fn get(&self) -> bool
    {
        let mut data = [0 as u8; 2];
        let pinbit = if self.pin <= 7 {
            1 << (self.pin + 8)
        } else if self.pin <= 17 && self.pin >= 10 {
            1 << (self.pin - 10)
        } else {
            panic!("invalid pin number {}", self.pin);
        };

        self.dev.lock().read(&[], &mut data).unwrap();

        let data_u16 = (data[1] as u16) | ((data[0] as u16) << 8);

        let v = data_u16 & pinbit != 0;
        v ^ self.invert
    }

    fn name(&self) -> &'static str
    {
        self.name
    }
}

unsafe impl<'a> Sync for PcfGpio<'a> {}
