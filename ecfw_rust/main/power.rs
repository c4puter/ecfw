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
use main::supplies;
use core::sync::atomic::*;

pub static VRM901: TwiDevice = TwiDevice::new(&twi::TWI0, 0x47);

/// Mutex used to lock power supply operations. External code should take this
/// mutex before changing power supply settings, and release it when the change
/// is complete and settled.
pub static POWER_MUTEX: mutex::Mutex<()> = mutex::Mutex::new(());

#[allow(unused)]
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum SupplyStatus {
    Down,
    Up,
    Transition,
    Error
}

pub trait Supply : Sync {
    /// Return the supply's name. Override the default if not wrapping a
    /// virtual supply.
    fn name(&self) -> &'static str;

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

    /// Bring this supply up. Will panic if any of its dependencies are not up.
    /// Does nothing if already up.
    ///
    /// No timing guarantees: may return before the transition is complete or
    /// may block.
    fn up(&self) -> Result<(), &'static str>;

    /// Bring this supply down. Will panic if any of its dependants are not
    /// down.
    /// Does nothing if already down.
    ///
    /// No timing guarantees: may return before the transition is complete or
    /// may block.
    fn down(&self) -> Result<(), &'static str>;

    /// Return a list of dependencies of this supply
    fn deps(&self) -> &[&Supply];

    /// Return the number of dependencies of this supply that are not up
    fn count_deps_not_up(&self) -> Result<usize,&'static str> {
        let mut count = 0usize;

        for &dep in self.deps() {
            if try!(dep.status()) != SupplyStatus::Up {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Return the number of dependants of this supply that are not down
    fn count_rev_deps_not_down(&self) -> Result<usize,&'static str> where Self: Sized {
        let mut count = 0usize;
        let self_ptr = self as *const Supply;

        for &supply in supplies::SUPPLY_TABLE {
            for &dep in supply.deps() {
                let dep_ptr = dep as *const Supply;

                if dep_ptr == self_ptr {
                    if try!(supply.status()) != SupplyStatus::Down {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }
}

/// Power supply section on the voltage regulator module
pub struct VrmSupply {
    vrm_id: u8,         // ID used by the VRM I2C interface
    disch: Option<(&'static Gpio, u32)>,
    set_state: AtomicBool,
    transitioning: AtomicBool,
    deps: &'static [&'static(Supply)],
    name: &'static str,
}

/// Power supply controlled by a single "enable" line on GPIO
pub struct GpioSwitchedSupply {
    gpio: &'static Gpio,
    disch: Option<(&'static Gpio, u32)>,
    wait_ticks: u32,    // Number of 1ms ticks to wait after switching
                        // to consider the supply settled
    deps: &'static [&'static(Supply)],
    name: &'static str,
}

impl VrmSupply {
    pub const fn new(
        name: &'static str,
        deps: &'static [&'static Supply],
        vrm_id: u8,
        disch: Option<(&'static Gpio, u32)>) -> VrmSupply
    {
        VrmSupply {
            vrm_id: vrm_id,
            disch: disch,
            set_state: ATOMIC_BOOL_INIT,
            transitioning: ATOMIC_BOOL_INIT,
            deps: deps,
            name: name,
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

    fn up(&self) -> Result<(), &'static str> {
        assert!(try!(self.count_deps_not_up()) == 0);

        match self.disch {
            Some((gpio, _wait)) => {
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

    fn down(&self) -> Result<(), &'static str> {
        assert!(try!(self.count_rev_deps_not_down()) == 0);

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

    fn deps(&self) -> &[&Supply] {
        &self.deps
    }

    fn name(&self) -> &'static str {
        &self.name
    }
}

impl GpioSwitchedSupply {
    pub const fn new(
        name: &'static str,
        deps: &'static [&'static Supply],
        gpio: &'static Gpio,
        wait_ticks: u32,
        disch: Option<(&'static Gpio, u32)>) -> GpioSwitchedSupply
    {
        GpioSwitchedSupply {
            gpio: gpio,
            disch: disch,
            wait_ticks: wait_ticks,
            deps: deps,
            name: name,
        }
    }
}

impl Supply for GpioSwitchedSupply {
    fn status(&self) -> Result<SupplyStatus, &'static str> {
        if self.gpio.get() { Ok(SupplyStatus::Up) } else { Ok(SupplyStatus::Down) }
    }

    fn up(&self) -> Result<(), &'static str> {
        assert!(try!(self.count_deps_not_up()) == 0);

        match self.disch {
            Some((disgpio, _wait)) => {
                disgpio.set(false);
            },
            None => ()
        }
        self.gpio.set(true);
        freertos::susp_safe_delay(self.wait_ticks);
        Ok(())
    }

    fn down(&self) -> Result<(), &'static str> {
        assert!(try!(self.count_rev_deps_not_down()) == 0);

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

    fn deps(&self) -> &[&Supply] {
        &self.deps
    }

    fn name(&self) -> &'static str {
        &self.name
    }
}


unsafe impl Sync for VrmSupply {}
unsafe impl Sync for GpioSwitchedSupply {}
