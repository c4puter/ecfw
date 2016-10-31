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

use rustsys::{freertos,mutex};
use hardware::twi;
use hardware::gpio::Gpio;
use hardware::twi::TwiDevice;
use core::sync::atomic::*;

pub static VRM901: TwiDevice = TwiDevice::new(&twi::TWI0, 0x47);

/// Mutex used to lock power supply operations. External code should take this
/// mutex before changing power supply settings, and release it when the change
/// is complete and settled.
pub static POWER_MUTEX: mutex::Mutex = mutex::Mutex::new();

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum SupplyStatus {
    Down,
    Up,
    Transition,
    Error
}

pub trait Supply {
    /// Return the supply's name. Override the default if not wrapping a
    /// virtual supply.
    fn name(&self) -> &'static str {
        self.getvirt().unwrap().name()
    }

    /// Check and return the supply status, or Err if an error occurred
    /// getting the status
    fn status(&self) -> Result<SupplyStatus, &'static str>;

    /// Wait until the status is 'status'. Times out and panics after one second.
    fn wait_status(&self, status: SupplyStatus) -> Result<(), &'static str> {
        let mut to_timeout = 1000;
        loop {
            match self.status() {
                Ok(s) => {
                    if s == status { return Ok(()); }
                    else {
                        if to_timeout > 0 {
                            to_timeout -= 1;
                            freertos::susp_safe_delay(1);
                        } else {
                            panic!("timeout waiting for supply {} state change: {:?} -> {:?}",
                                   self.name(), s, status)
                        }
                    }
                },
                Err(e) => { return Err(e); }
            }
        }
    }

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
        let ref virt = self.getvirt().unwrap();
        match virt.refcount_up() {
            Ok(true) => {
                match virt.wait_status(SupplyStatus::Up) {
                    Ok(_) => (),
                    Err(e) => { return Err(e); }
                };
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
        let ref virt = self.getvirt().unwrap();
        let rc = virt.refcount();
        if rc == 1 {
            match self.disable() {
                Ok(_) => {
                    match self.wait_status(SupplyStatus::Down) {
                        Ok(_) => (),
                        Err(e) => { return Err(e); }
                    }
                    match virt.refcount_down() {
                        Ok(_) => Ok(true),
                        Err(e) => Err(e),
                    }
                },
                Err(e) => Err(e),
            }
        } else if rc > 1 {
            match virt.refcount_down() {
                Ok(_) => Ok(false),
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
    virt: VirtualSupply,
    vrm_id: u8,         // ID used by the VRM I2C interface
    disch: Option<(&'static Gpio, u32)>,
    set_state: AtomicBool,
    transitioning: AtomicBool,
}

/// Power supply controlled by a single "enable" line on GPIO
pub struct GpioSwitchedSupply {
    virt: VirtualSupply,
    gpio: &'static Gpio,
    disch: Option<(&'static Gpio, u32)>,
    wait_ticks: u32,    // Number of 1ms ticks to wait after switching
                        // to consider the supply settled
}

/// Power state (virtual supply, only has dependencies). This is also used by
/// the real supply objects for dependency handling, to keep that code in one
/// place.
pub struct VirtualSupply {
    name: &'static str,
    deps: &'static [&'static Supply],
    refcount: AtomicUsize,
}

impl VrmSupply {
    pub const fn new(
        name: &'static str,
        deps: &'static [&'static Supply],
        vrm_id: u8) -> VrmSupply
    {
        VrmSupply {
            virt: VirtualSupply::new(name, deps),
            vrm_id: vrm_id,
            disch: None,
            set_state: ATOMIC_BOOL_INIT,
            transitioning: ATOMIC_BOOL_INIT,
        }
    }

    pub const fn new_disch(
        name: &'static str,
        deps: &'static [&'static Supply],
        vrm_id: u8,
        disch: &'static Gpio,
        dischwait: u32) -> VrmSupply
    {
        VrmSupply {
            virt: VirtualSupply::new(name, deps),
            vrm_id: vrm_id,
            disch: Some((disch, dischwait)),
            set_state: ATOMIC_BOOL_INIT,
            transitioning: ATOMIC_BOOL_INIT,
        }
    }

    pub const CTRL_BIT_ENABLED: u8 =    1u8 << 0;
    pub const CTRL_BIT_POWER_GOOD: u8 = 1u8 << 1;
}

impl Supply for VrmSupply {
    fn status(&self) -> Result<SupplyStatus, &'static str> {
        let up_bits = VrmSupply::CTRL_BIT_ENABLED | VrmSupply::CTRL_BIT_POWER_GOOD;
        let mut buf = [0u8; 1];
        let up = match VRM901.read(&[self.vrm_id], &mut buf) {
            Ok(_) => {
                buf[0] & up_bits == up_bits
            },
            Err(e) => { return Err(e.description());}
        };

        if self.transitioning.load(Ordering::Relaxed) {
            if up == self.set_state.load(Ordering::Relaxed) {
                self.transitioning.store(false, Ordering::Relaxed);
                if up { Ok(SupplyStatus::Up) } else { Ok(SupplyStatus::Down) }
            } else {
                Ok(SupplyStatus::Transition)
            }
        } else {
            if up { Ok(SupplyStatus::Up) } else { Ok(SupplyStatus::Down) }
        }
    }

    fn enable(&self) -> Result<(), &'static str> {
        match self.disch {
            Some((gpio, wait)) => {
                gpio.set(false);
            },
            None => ()
        }
        match VRM901.write(&[self.vrm_id], &[VrmSupply::CTRL_BIT_ENABLED]) {
            Ok(_) => {
                self.set_state.store(true, Ordering::Relaxed);
                self.transitioning.store(true, Ordering::Relaxed);
                Ok(()) },
            Err(e) => Err(e.description())
        }
    }

    fn disable(&self) -> Result<(), &'static str> {
        match VRM901.write(&[self.vrm_id], &[0]) {
            Ok(_) => {
                self.set_state.store(false, Ordering::Relaxed);
                self.transitioning.store(false, Ordering::Relaxed);
                match self.disch {
                    Some((gpio, wait)) => {
                        gpio.set(true);
                        freertos::susp_safe_delay(wait);
                    },
                    None => ()
                };
                Ok(()) },
            Err(e) => Err(e.description())
        }
    }

    fn getvirt(&self) -> Option<&VirtualSupply> {
        Some(&self.virt)
    }
}

impl GpioSwitchedSupply {
    pub const fn new(
        name: &'static str,
        deps: &'static [&'static Supply],
        gpio: &'static Gpio,
        wait_ticks: u32 ) -> GpioSwitchedSupply
    {
        GpioSwitchedSupply {
            virt: VirtualSupply::new(name, deps),
            gpio: gpio,
            disch: None,
            wait_ticks: wait_ticks,
        }
    }

    pub const fn new_disch(
        name: &'static str,
        deps: &'static [&'static Supply],
        gpio: &'static Gpio,
        wait_ticks: u32,
        disch: &'static Gpio,
        dischwait: u32) -> GpioSwitchedSupply
    {
        GpioSwitchedSupply {
            virt: VirtualSupply::new(name, deps),
            gpio: gpio,
            disch: Some((disch, dischwait)),
            wait_ticks: wait_ticks,
        }
    }
}

impl Supply for GpioSwitchedSupply {
    fn status(&self) -> Result<SupplyStatus, &'static str> {
        if self.gpio.get() { Ok(SupplyStatus::Up) } else { Ok(SupplyStatus::Down) }
    }

    fn enable(&self) -> Result<(), &'static str> {
        match self.disch {
            Some((disgpio, wait)) => {
                disgpio.set(false);
            },
            None => ()
        }
        self.gpio.set(true);
        freertos::susp_safe_delay(self.wait_ticks);
        Ok(())
    }

    fn disable(&self) -> Result<(), &'static str> {
        let mut max_wait = self.wait_ticks;

        self.gpio.set(false);
        match self.disch {
            Some((disgpio, wait)) => {
                disgpio.set(true);
                if wait > max_wait { max_wait = wait; }
            },
            None => ()
        };
        freertos::susp_safe_delay(max_wait);
        Ok(())
    }

    fn getvirt(&self) -> Option<&VirtualSupply> {
        Some(&self.virt)
    }
}

impl VirtualSupply {
    pub const fn new(name: &'static str, deps: &'static [&'static Supply]) -> VirtualSupply {
        VirtualSupply {
            name: name,
            deps: deps,
            refcount: ATOMIC_USIZE_INIT,
        }
    }
}

impl Supply for VirtualSupply {
    fn name(&self) -> &'static str {
        self.name
    }

    fn status(&self) -> Result<SupplyStatus, &'static str> {
        // Up: all deps are up
        // Down: all deps are down
        // Transition: any dep is transitioning
        // Error: mix of up and down, but no transition
        let mut all_up = true;
        let mut all_down = true;
        let mut any_transition = false;
        let mut any_error = false;

        for supply in self.deps.iter() {
            match supply.status() {
                Ok(SupplyStatus::Up) => { all_down = false; },
                Ok(SupplyStatus::Down) => { all_up = false; },
                Ok(SupplyStatus::Transition) => { any_transition = true; all_down = false; all_up = false; },
                Ok(SupplyStatus::Error) => { any_error = true; },
                Err(e) => { return Err(e) }
            }
        }

        match (all_up, all_down, any_transition, any_error) {
            (true, _, _, false) => Ok(SupplyStatus::Up),
            (_, true, _, false) => Ok(SupplyStatus::Down),
            (false, false, true, false) => Ok(SupplyStatus::Transition),
            (_, _, _, true) => Ok(SupplyStatus::Error),
            (false, false, false, false) => Ok(SupplyStatus::Error),
        }
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
