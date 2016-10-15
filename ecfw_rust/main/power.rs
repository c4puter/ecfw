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

use rustsys::{freertos,smutex};
use hardware::twi;
use hardware::gpio::Gpio;
use hardware::twi::TwiDevice;
use core::sync::atomic::*;

static VRM901: TwiDevice = TwiDevice::new(twi::twi0, 0x47);

/// Mutex used to lock power supply operations. External code should take this
/// mutex before changing power supply settings, and release it when the change
/// is complete and settled.
pub static POWER_MUTEX: smutex::StaticMutex = smutex::StaticMutex::new();

pub trait Supply {
    /// Return the supply's name. Override the default if not wrapping a
    /// virtual supply.
    fn name(&self) -> &'static str {
        self.getvirt().unwrap().name()
    }

    /// Check if the supply is up.
    /// Return Ok(true) if up
    /// Return Ok(false) if down
    /// Return Err(&'static str) if an error occurred getting the state
    fn is_up(&self) -> Result<bool, &'static str>;

    /// Check if the supply has failed (went down after coming up, or took
    /// too long to come up)
    ///
    /// Return Ok(()) if the supply has not failed
    /// Return Err(&'static str) with a failure message if it has
    fn is_failed(&self) -> Result<(), &'static str>;

    /// Add to the dependency reference count. If the reference count exceeds
    /// zero, enable the supply. If the reference count falls to zero, disable
    /// it and discharge it if possible.
    ///
    /// Reference counting need not be threadsafe by itself; use the global
    /// power control mutex instead. Internal power.rs functions should not
    /// touch the mutex.
    ///
    /// Return Ok(true) if the supply state was changed
    /// Return Ok(false) if no change occurred
    /// Return Err(&'static str) if an error occurred changing the supply state
    ///
    /// Default implementation assumes a wrapped virtual.
    fn refcount_up(&self) -> Result<bool, &'static str> {
        match self.getvirt().unwrap().refcount_up() {
            Ok(true) => {
                match self.enable() {
                    Ok(_) => Ok(true),
                    Err(e) => Err(e),
                }
            },
            Ok(false) => Ok(false),
            Err(e) => Err(e)
        }
    }

    /// Subtract from the dependency reference count. If the reference count
    /// falls to zero, disable the supply and discharge it if possible.
    ///
    /// Reference counting need not be threadsafe by itself; use the global
    /// power control mutex instead. Internal power.rs functions should not
    /// touch the mutex.
    ///
    /// Return Ok(true) if the supply state was changed
    /// Return Ok(false) if no change occurred
    /// Return Err(&'static str) if an error occurred changing the supply state
    ///
    /// Default implementation assumes a wrapped virtual.
    fn refcount_down(&self) -> Result<bool, &'static str> {
        if self.getvirt().unwrap().refcount() == 1 {
            match self.disable() {
                Ok(_) => {
                    match self.getvirt().unwrap().refcount_down() {
                        Ok(_) => Ok(true),
                        Err(e) => Err(e),
                    }
                },
                Err(e) => Err(e),
            }
        } else {
            Ok(false)
        }
    }

    /// Return the reference count. Override the default if not wrapping a
    /// virtual supply.
    fn refcount(&self) -> usize {
        self.getvirt().unwrap().refcount()
    }

    fn getvirt(&self) -> Option<&VirtualSupply>;

    /// Enable only this supply, assuming dependencies are up. Only used
    /// by the default refcount_up() implementation.
    fn enable(&self) -> Result<(), &'static str>;

    /// Disable only this supply, assuming dependencies are being tracked.
    /// Only used by the default refcount_down() implementation.
    fn disable(&self) -> Result<(), &'static str>;
}

/// Power supply section on the voltage regulator module
pub struct VrmSupply {
    pub virt: VirtualSupply,
    pub vrm_id: u8,         // ID used by the VRM I2C interface
    pub was_up: AtomicBool, // Tracks whether the supply has been up before,
                            // to determine failure
}

/// Power supply controlled by a single "enable" line on GPIO
pub struct GpioSwitchedSupply {
    pub virt: VirtualSupply,
    pub gpio: &'static Gpio,
    pub wait_ticks: u32,    // Number of 1ms ticks to wait after switching
                            // to consider the supply settled
}

/// Power state (virtual supply, only has dependencies). This is also used by
/// the real supply objects for dependency handling, to keep that code in one
/// place.
pub struct VirtualSupply {
    pub name: &'static str,
    pub deps: &'static [&'static Supply],
    pub refcount: AtomicUsize,
}

impl VrmSupply {
    pub const CTRL_BIT_ENABLED: u8 =    1u8 << 0;
    pub const CTRL_BIT_POWER_GOOD: u8 = 1u8 << 1;
}

impl Supply for VrmSupply {
    fn is_up(&self) -> Result<bool, &'static str> {
        let up_bits = VrmSupply::CTRL_BIT_ENABLED | VrmSupply::CTRL_BIT_POWER_GOOD;
        let mut buf = [0u8; 1];
        match VRM901.read(&[self.vrm_id], &mut buf) {
            Ok(_) => {
                let was_up = buf[0] & up_bits == up_bits;
                self.was_up.store(was_up, Ordering::Relaxed);
                Ok(was_up)
            },
            Err(e) => Err(e.description())
        }
    }

    fn is_failed(&self) -> Result<(), &'static str> {
        match self.is_up() {
            Ok(is_up) => {
                if !is_up && self.was_up.load(Ordering::Relaxed) {
                    Err("supply voltage has dropped")
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(e)
        }
    }

    fn enable(&self) -> Result<(), &'static str> {
        match VRM901.write(&[self.vrm_id], &[VrmSupply::CTRL_BIT_ENABLED]) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.description())
        }
    }

    fn disable(&self) -> Result<(), &'static str> {
        match VRM901.write(&[self.vrm_id], &[0]) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.description())
        }
    }

    fn getvirt(&self) -> Option<&VirtualSupply> {
        Some(&self.virt)
    }
}

impl Supply for GpioSwitchedSupply {
    fn is_up(&self) -> Result<bool, &'static str> {
        Ok(self.gpio.get())
    }

    fn is_failed(&self) -> Result<(), &'static str> {
        Ok(())
    }

    fn enable(&self) -> Result<(), &'static str> {
        self.gpio.set(true);
        freertos::delay(self.wait_ticks);
        Ok(())
    }

    fn disable(&self) -> Result<(), &'static str> {
        self.gpio.set(false);
        freertos::delay(self.wait_ticks);
        Ok(())
    }

    fn getvirt(&self) -> Option<&VirtualSupply> {
        Some(&self.virt)
    }
}

impl Supply for VirtualSupply {
    fn name(&self) -> &'static str {
        self.name
    }

    fn is_up(&self) -> Result<bool, &'static str> {
        for supply in self.deps.iter() {
            match supply.is_up() {
                Ok(true) => (),
                Ok(false) => { return Ok(false); }
                Err(e) => { return Err(e); }
            }
        }
        Ok(true)
    }

    fn is_failed(&self) -> Result<(), &'static str> {
        for supply in self.deps.iter() {
            match supply.is_failed() {
                Ok(()) => (),
                Err(e) => { return Err(e); }
            }
        }
        Ok(())
    }

    fn refcount_up(&self) -> Result<bool, &'static str> {
        let lastcount = self.refcount.fetch_add(1, Ordering::Relaxed);
        if lastcount == 0 {
            for supply in self.deps.iter() {
                match supply.refcount_up() {
                    Ok(_) => (),
                    Err(e) => { return Err(e); }
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn refcount_down(&self) -> Result<bool, &'static str> {
        let lastcount = self.refcount.fetch_sub(1, Ordering::Relaxed);
        if lastcount == 1 {
            for supply in self.deps.iter() {
                match supply.refcount_down() {
                    Ok(_) => (),
                    Err(e) => { return Err(e); }
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn refcount(&self) -> usize {
        self.refcount.load(Ordering::Relaxed)
    }

    fn getvirt(&self) -> Option<&VirtualSupply> {
        None
    }

    fn enable(&self) -> Result<(), &'static str> {
        Err("enable/disable of virtual supply")
    }

    fn disable(&self) -> Result<(), &'static str> {
        Err("enable/disable of virtual supply")
    }
}

unsafe impl Sync for VrmSupply {}
unsafe impl Sync for GpioSwitchedSupply {}
unsafe impl Sync for VirtualSupply {}
