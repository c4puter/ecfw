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

use os::Mutex;
use messages::*;
extern crate bindgen_mcu;
pub type TwiHandle = u32;
use core::sync::atomic::*;

#[repr(u32)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum TwiResultCode {
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

fn to_stdresult(code: TwiResultCode) -> StdResult
{
    match code {
        TwiResultCode::Success          => Ok(()),
        TwiResultCode::InvalidArgument  => Err(ERR_TWI_INVALID),
        TwiResultCode::ArbitrationLost  => Err(ERR_TWI_ARBITRATION),
        TwiResultCode::NoChipFound      => Err(ERR_TWI_NOTFOUND),
        TwiResultCode::ReceiveOverrun   => Err(ERR_TWI_RXOVF),
        TwiResultCode::ReceiveNack      => Err(ERR_TWI_RXNACK),
        TwiResultCode::SendOverrun      => Err(ERR_TWI_TXOVF),
        TwiResultCode::SendNack         => Err(ERR_TWI_TXNACK),
        TwiResultCode::Busy             => Err(ERR_BUSY),
        TwiResultCode::ErrorTimeout     => Err(ERR_TIMEOUT),
    }
}

#[repr(C)]
struct TwiOptions {
    master_clk: u32,
    speed: u32,
    chip: u8,
    smbus: bool,
}

#[repr(C)]
struct TwiPacket {
    addr: [u8; 3],
    addr_length: u32,
    buffer: *mut u8,
    length: u32,
    chip: u8,
}

extern "C" {
    fn twi_master_init(p_twi: TwiHandle, p_opt: *const TwiOptions) -> TwiResultCode;
    fn twi_probe(p_twi: TwiHandle, uc_slave_addr: u8) -> TwiResultCode;
    fn twi_master_read(p_twi: TwiHandle, p_packet: *mut TwiPacket) -> TwiResultCode;
    fn twi_master_write(p_twi: TwiHandle, p_packet: *mut TwiPacket) -> TwiResultCode;
}

pub struct Twi {
    p_twi: TwiHandle,
    mutex: Mutex<()>,
    initialized: AtomicBool,
}

pub struct TwiDevice {
    pub twi: &'static Twi,
    pub addr: u8,
}

/// Threadsafe wrapper around TWI peripheral. This must be initialized before
/// use; use before init() or a double-init() will panic.
impl Twi {
    pub const fn new(p_twi: TwiHandle) -> Twi {
        Twi {
            p_twi: p_twi,
            mutex: Mutex::new(()),
            initialized: ATOMIC_BOOL_INIT,
        }
    }

    /// Initialize the TWI; panic if double-initialized.
    pub fn init(&self, speed: u32) -> StdResult {
        let was_initialized = self.initialized.swap(true, Ordering::Relaxed);
        if was_initialized {
            panic!("TWI: double init()");
        }

        let opts = TwiOptions {
            master_clk: unsafe{bindgen_mcu::mcu_get_peripheral_hz()},
            speed: speed,
            chip: 0,
            smbus: false };
        let rc = unsafe{twi_master_init(self.p_twi, &opts)};
        to_stdresult(rc)
    }

    /// Test if a device answers a given address
    pub fn probe(&self, addr: u8) -> Result<bool,Error> {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("TWI: use before init()");
        }

        let _lock = self.mutex.lock();
        let rc = unsafe{twi_probe(self.p_twi, addr)};
        match rc {
            TwiResultCode::Success      => Ok(true),
            TwiResultCode::ReceiveNack  => Ok(false),
            _                           => Err(to_stdresult(rc).unwrap_err())
        }
    }

    /// Read from 'addr' at 'location' into 'buffer'.
    /// addr:       I2C address
    /// location:   register address in the chip, up to 3 bytes
    /// buffer:     buffer to receive. Will receive buffer.len() bytes
    pub fn read(&self, addr: u8, location: &[u8], buffer: &mut [u8]) -> StdResult {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("TWI: use before init()");
        }
        let _lock = self.mutex.lock();
        if location.len() > 3 {
            return Err(ERR_TWI_INVALID);
        }
        let mut packet = TwiPacket {
            addr: [0; 3],
            addr_length: location.len() as u32,
            buffer: buffer.as_mut_ptr(),
            length: buffer.len() as u32,
            chip: addr,
        };
        (&mut packet.addr[0..location.len()]).clone_from_slice(&location);
        let rc = unsafe{twi_master_read(self.p_twi, &mut packet)};
        to_stdresult(rc)
    }

    /// Write to 'addr' at 'location' from 'buffer'.
    /// addr:       I2C address
    /// location:   register address in the chip, up to 3 bytes
    /// buffer:     buffer to write. Will write buffer.len() bytes
    pub fn write(&self, addr: u8, location: &[u8], buffer: &[u8]) -> StdResult {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("TWI: use before init()");
        }
        let _lock = self.mutex.lock();
        if location.len() > 3 {
            return Err(ERR_TWI_INVALID)
        }
        let mut packet = TwiPacket {
            addr: [0; 3],
            addr_length: location.len() as u32,
            buffer: buffer.as_ptr() as *mut u8,
            length: buffer.len() as u32,
            chip: addr,
        };
        (&mut packet.addr[0..location.len()]).clone_from_slice(&location);
        let rc = unsafe{twi_master_write(self.p_twi, &mut packet)};
        to_stdresult(rc)
    }
}

impl TwiDevice {
    pub const fn new(twi: &'static Twi, addr: u8) -> TwiDevice {
        TwiDevice {
            twi: twi,
            addr: addr,
        }
    }

    /// Test if the device answers its address
    #[allow(dead_code)]
    pub fn probe(&mut self) -> Result<bool, Error> {
        self.twi.probe(self.addr)
    }

    /// Read from 'location' into 'buffer'
    /// location:   register address in the chip, zero to three bytes
    /// buffer:     buffer to receive. Will receive buffer.len() bytes
    pub fn read(&mut self, location: &[u8], buffer: &mut [u8]) -> StdResult
    {
        self.twi.read(self.addr, location, buffer)
    }

    /// Write to 'location' from 'buffer'.
    /// location:   register address in the chip, zero to three bytes
    /// buffer:     buffer to write. Will write buffer.len() bytes
    pub fn write(&mut self, location: &[u8], buffer: &[u8]) -> StdResult
    {
        self.twi.write(self.addr, location, buffer)
    }
}