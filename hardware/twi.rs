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

extern crate rust_support;
extern crate bindgen_mcu;
extern crate freertos;
extern crate smutex;
type TwiHandle = u32;
use core::fmt;

pub const TWI0_HANDLE: TwiHandle = 0x40018000;
pub const TWI1_HANDLE: TwiHandle = 0x4001C000;

static mut TWI0_: Twi = Twi { p_twi: TWI0_HANDLE, mutex: None };
static mut TWI1_: Twi = Twi { p_twi: TWI1_HANDLE, mutex: None };

pub fn twi0() -> &'static Twi { return unsafe{&TWI0_}; }
pub fn twi1() -> &'static Twi { return unsafe{&TWI1_}; }

pub unsafe fn twi0_init(speed: u32) -> Result<(),TwiResult> { TWI0_.init(speed) }
pub unsafe fn twi1_init(speed: u32) -> Result<(),TwiResult> { TWI1_.init(speed) }

#[repr(u32)]
#[derive(Debug)]
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

#[repr(C)]
#[derive(Debug)]
pub struct TwiResult {
    code: TwiResultCode
}

impl TwiResult {
    pub fn description(&self) -> &'static str {
        return match self.code {
            TwiResultCode::Success          => "success",
            TwiResultCode::InvalidArgument  => "invalid argument",
            TwiResultCode::ArbitrationLost  => "arbitration lost",
            TwiResultCode::NoChipFound      => "no chip found",
            TwiResultCode::ReceiveOverrun   => "receive overrun",
            TwiResultCode::ReceiveNack      => "receive NACK",
            TwiResultCode::SendOverrun      => "send overrun",
            TwiResultCode::SendNack         => "send NACK",
            TwiResultCode::Busy             => "busy",
            TwiResultCode::ErrorTimeout     => "timeout",
        }
    }
}

impl fmt::Display for TwiResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TwiResult({})", self.description())
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
    fn twi_master_init(p_twi: TwiHandle, p_opt: *const TwiOptions) -> TwiResult;
    fn twi_probe(p_twi: TwiHandle, uc_slave_addr: u8) -> TwiResult;
    fn twi_master_read(p_twi: TwiHandle, p_packet: *mut TwiPacket) -> TwiResult;
    fn twi_master_write(p_twi: TwiHandle, p_packet: *mut TwiPacket) -> TwiResult;
}

#[derive(Copy,Clone)]
pub struct Twi {
    p_twi: TwiHandle,
    mutex: Option<freertos::Mutex>,
}

pub struct TwiDevice {
    pub twi: fn() -> &'static Twi,
    pub addr: u8,
    pub mutex: smutex::StaticMutex,
}

impl Twi {
    pub fn init(&mut self, speed: u32) -> Result<(),TwiResult> {
        self.mutex = Some(freertos::Mutex::new());
        let opts = TwiOptions {
            master_clk: unsafe{bindgen_mcu::mcu_get_peripheral_hz()},
            speed: speed,
            chip: 0,
            smbus: false };
        let rc = unsafe{twi_master_init(self.p_twi, &opts)};
        return match rc.code {
            TwiResultCode::Success  => Ok(()),
            _                       => Err(rc)
        };
    }

    /// Test if a device answers a given address
    pub fn probe(&self, addr: u8) -> Result<bool,TwiResult> {
        let _lock = self.mutex.unwrap().lock(1000).unwrap();
        let rc = unsafe{twi_probe(self.p_twi, addr)};
        return match rc.code {
            TwiResultCode::Success      => Ok(true),
            TwiResultCode::ReceiveNack  => Ok(false),
            _                           => Err(rc)
        };
    }

    /// Read from 'addr' at 'location' into 'buffer'.
    /// addr:       I2C address
    /// location:   register address in the chip, up to 3 bytes
    /// buffer:     buffer to receive. Will receive buffer.len() bytes
    pub fn read(&self, addr: u8, location: &[u8], buffer: &mut [u8]) -> Result<(), TwiResult> {
        let _lock = self.mutex.unwrap().lock(1000).unwrap();
        if location.len() > 3 {
            return Err(TwiResult{code: TwiResultCode::InvalidArgument});
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
        return match rc.code {
            TwiResultCode::Success      => Ok(()),
            _                           => Err(rc),
        };
    }

    /// Write to 'addr' at 'location' from 'buffer'.
    /// addr:       I2C address
    /// location:   register address in the chip, up to 3 bytes
    /// buffer:     buffer to write. Will write buffer.len() bytes
    pub fn write(&self, addr: u8, location: &[u8], buffer: &[u8]) -> Result<(), TwiResult> {
        let _lock = self.mutex.unwrap().lock(1000).unwrap();
        if location.len() > 3 {
            return Err(TwiResult{code: TwiResultCode::InvalidArgument});
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
        return match rc.code {
            TwiResultCode::Success      => Ok(()),
            _                           => Err(rc),
        };
    }
}

impl TwiDevice {
    /// Obtain and return a lock on this device. This is an RAII lock that will
    /// be released when it goes out of scope.
    pub fn lock(&'static self) -> smutex::StaticMutexLock {
        self.mutex.lock()
    }

    /// Test if the device answers its address
    pub fn probe(&'static self) -> Result<bool, TwiResult> {
        (self.twi)().probe(self.addr)
    }

    /// Read from 'location' into 'buffer'
    /// location:   register address in the chip, zero to three bytes
    /// buffer:     buffer to receive. Will receive buffer.len() bytes
    pub fn read(&'static self, location: &[u8], buffer: &mut [u8])
            -> Result<(), TwiResult>
    {
        (self.twi)().read(self.addr, location, buffer)
    }

    /// Write to 'location' from 'buffer'.
    /// location:   register address in the chip, zero to three bytes
    /// buffer:     buffer to write. Will write buffer.len() bytes
    pub fn write(&'static self, location: &[u8], buffer: &[u8])
            -> Result<(), TwiResult>
    {
        (self.twi)().write(self.addr, location, buffer)
    }
}
