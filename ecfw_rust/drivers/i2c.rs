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

//! On-chip I2C driver (wrapper around Atmel ASF's TWI)

use os::Mutex;
use messages::*;
extern crate bindgen_mcu;
pub type I2CHandle = u32;
use core::sync::atomic::*;

#[repr(u32)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum I2CResultCode {
    Success         = 0,
    InvalidArgument = 1,
    ArbitrationLost = 2,
    NoChipFound     = 3,
    ReceiveOverrun  = 4,
    ReceiveNack     = 5,
    SendOverrun     = 6,
    SendNack        = 7,
    Busy            = 8,
    ErrorTimeout    = 9,
}

fn to_stdresult(code: I2CResultCode) -> StdResult
{
    match code {
        I2CResultCode::Success          => Ok(()),
        I2CResultCode::InvalidArgument  => Err(ERR_I2C_INVALID),
        I2CResultCode::ArbitrationLost  => Err(ERR_I2C_ARBITRATION),
        I2CResultCode::NoChipFound      => Err(ERR_I2C_NOTFOUND),
        I2CResultCode::ReceiveOverrun   => Err(ERR_I2C_RXOVF),
        I2CResultCode::ReceiveNack      => Err(ERR_I2C_RXNACK),
        I2CResultCode::SendOverrun      => Err(ERR_I2C_TXOVF),
        I2CResultCode::SendNack         => Err(ERR_I2C_TXNACK),
        I2CResultCode::Busy             => Err(ERR_BUSY),
        I2CResultCode::ErrorTimeout     => Err(ERR_TIMEOUT),
    }
}

#[repr(C)]
struct I2COptions {
    master_clk: u32,
    speed: u32,
    chip: u8,
    smbus: bool,
}

#[repr(C)]
struct I2CPacket {
    addr: [u8; 3],
    addr_length: u32,
    buffer: *mut u8,
    length: u32,
    chip: u8,
}

extern "C" {
    fn twi_master_init(
        p_i2c: I2CHandle,
        p_opt: *const I2COptions,
    ) -> I2CResultCode;
    fn twi_probe(p_i2c: I2CHandle, uc_slave_addr: u8) -> I2CResultCode;
    fn twi_master_read(
        p_i2c: I2CHandle,
        p_packet: *mut I2CPacket,
    ) -> I2CResultCode;
    fn twi_master_write(
        p_i2c: I2CHandle,
        p_packet: *mut I2CPacket,
    ) -> I2CResultCode;
}

pub struct I2C {
    p_i2c: I2CHandle,
    mutex: Mutex<()>,
    initialized: AtomicBool,
}

pub struct I2CDevice<'a> {
    pub i2c: &'a I2C,
    pub addr: u8,
}

/// Threadsafe wrapper around I2C peripheral. This must be initialized before
/// use; use before init() or a double-init() will panic.
impl I2C {
    pub const fn new(p_i2c: I2CHandle) -> I2C
    {
        I2C {
            p_i2c: p_i2c,
            mutex: Mutex::new(()),
            initialized: ATOMIC_BOOL_INIT,
        }
    }

    /// Initialize the I2C; panic if double-initialized.
    pub fn init(&self, speed: u32) -> StdResult
    {
        let was_initialized = self.initialized.swap(true, Ordering::Relaxed);
        if was_initialized {
            panic!("I2C: double init()");
        }

        let opts = I2COptions {
            master_clk: unsafe { bindgen_mcu::mcu_get_peripheral_hz() },
            speed: speed,
            chip: 0,
            smbus: false,
        };
        let rc = unsafe { twi_master_init(self.p_i2c, &opts) };
        to_stdresult(rc)
    }

    /// Test if a device answers a given address
    pub fn probe(&self, addr: u8) -> Result<bool, Error>
    {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("I2C: use before init()");
        }

        let _lock = self.mutex.lock();
        let rc = unsafe { twi_probe(self.p_i2c, addr) };
        match rc {
            I2CResultCode::Success => Ok(true),
            I2CResultCode::ReceiveNack => Ok(false),
            _ => Err(to_stdresult(rc).unwrap_err()),
        }
    }

    /// Read from 'addr' at 'location' into 'buffer'.
    /// addr:       I2C address
    /// location:   register address in the chip, up to 3 bytes
    /// buffer:     buffer to receive. Will receive buffer.len() bytes
    pub fn read(
        &self,
        addr: u8,
        location: &[u8],
        buffer: &mut [u8],
    ) -> StdResult
    {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("I2C: use before init()");
        }
        let _lock = self.mutex.lock();
        if location.len() > 3 {
            return Err(ERR_I2C_INVALID);
        }
        let mut packet = I2CPacket {
            addr: [0; 3],
            addr_length: location.len() as u32,
            buffer: buffer.as_mut_ptr(),
            length: buffer.len() as u32,
            chip: addr,
        };
        (&mut packet.addr[0 .. location.len()]).clone_from_slice(
            &location,
        );
        let rc = unsafe { twi_master_read(self.p_i2c, &mut packet) };
        to_stdresult(rc)
    }

    /// Write to 'addr' at 'location' from 'buffer'.
    /// addr:       I2C address
    /// location:   register address in the chip, up to 3 bytes
    /// buffer:     buffer to write. Will write buffer.len() bytes
    pub fn write(&self, addr: u8, location: &[u8], buffer: &[u8]) -> StdResult
    {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("I2C: use before init()");
        }
        let _lock = self.mutex.lock();
        if location.len() > 3 {
            return Err(ERR_I2C_INVALID);
        }
        let mut packet = I2CPacket {
            addr: [0; 3],
            addr_length: location.len() as u32,
            buffer: buffer.as_ptr() as *mut u8,
            length: buffer.len() as u32,
            chip: addr,
        };
        (&mut packet.addr[0 .. location.len()]).clone_from_slice(
            &location,
        );
        let rc = unsafe { twi_master_write(self.p_i2c, &mut packet) };
        to_stdresult(rc)
    }
}

impl<'a> I2CDevice<'a> {
    pub const fn new(i2c: &'a I2C, addr: u8) -> I2CDevice<'a>
    {
        I2CDevice {
            i2c: i2c,
            addr: addr,
        }
    }

    /// Test if the device answers its address
    #[allow(dead_code)]
    pub fn probe(&mut self) -> Result<bool, Error>
    {
        self.i2c.probe(self.addr)
    }

    /// Read from 'location' into 'buffer'
    /// location:   register address in the chip, zero to three bytes
    /// buffer:     buffer to receive. Will receive buffer.len() bytes
    pub fn read(&mut self, location: &[u8], buffer: &mut [u8]) -> StdResult
    {
        self.i2c.read(self.addr, location, buffer)
    }

    /// Write to 'location' from 'buffer'.
    /// location:   register address in the chip, zero to three bytes
    /// buffer:     buffer to write. Will write buffer.len() bytes
    pub fn write(&mut self, location: &[u8], buffer: &[u8]) -> StdResult
    {
        self.i2c.write(self.addr, location, buffer)
    }
}
