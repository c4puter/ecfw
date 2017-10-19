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

use os;
use drivers::gpio::Gpio;
use devices::supplies;
use devices::twi::VRM901;
use messages::*;
use core::sync::atomic::*;

/// Mutex used to lock power supply operations. External code should take this
/// mutex before changing power supply settings, and release it when the change
/// is complete and settled.
pub static POWER_MUTEX: os::Mutex<()> = os::Mutex::new(());

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SupplyStatus {
    Down,
    Up,
    Transition,
    Error,
}

pub trait Supply: Sync {
    /// Return the supply's name. Override the default if not wrapping a
    /// virtual supply.
    fn name(&self) -> &'static str;

    /// Check and return the supply status, or Err if an error occurred
    /// getting the status
    fn status(&self) -> Result<SupplyStatus, Error>;

    /// Wait until the status is 'status'. Times out and panics after one
    /// second.
    fn wait_status(&self, status: SupplyStatus) -> StdResult
    {
        let mut to_timeout = 1000;
        loop {
            match self.status() {
                Ok(s) => {
                    if s == status {
                        return Ok(());
                    } else {
                        if to_timeout > 0 {
                            to_timeout -= 1;
                            os::susp_safe_delay(1);
                        } else {
                            panic!(
                                "timeout waiting for supply {} state change: \
                                 {:?} -> {:?}",
                                self.name(),
                                s,
                                status
                            )
                        }
                    }
                },
                Err(e) => {
                    return Err(e);
                },
            }
        }
    }

    /// Bring this supply up. Will panic if any of its dependencies are not
    /// up. Does nothing if already up.
    ///
    /// No timing guarantees: may return before the transition is complete
    /// or may block.
    fn up(&self) -> StdResult;

    /// Bring this supply down. Will panic if any of its dependants are not
    /// down. Does nothing if already down.
    ///
    /// No timing guarantees: may return before the transition is complete
    /// or may block.
    fn down(&self) -> StdResult;

    /// Return a list of dependencies of this supply
    fn deps(&self) -> &[&Supply];

    /// Return the number of dependencies of this supply that are not up
    fn count_deps_not_up(&self) -> Result<usize, Error>
    {
        let mut count = 0usize;

        for &dep in self.deps() {
            if dep.status()? != SupplyStatus::Up {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Return the number of dependants of this supply that are not down
    fn count_rev_deps_not_down(&self) -> Result<usize, Error>
    where
        Self: Sized,
    {
        let mut count = 0usize;
        let self_ptr = self as *const Supply;

        for &supply in supplies::SUPPLY_TABLE {
            for &dep in supply.deps() {
                let dep_ptr = dep as *const Supply;

                if dep_ptr == self_ptr {
                    if supply.status()? != SupplyStatus::Down {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }
}

/// Power supply section on the voltage regulator module
pub struct VrmSupply<'a> {
    vrm_id: u8, // ID used by the VRM I2C interface
    disch: Option<(&'a Gpio, u32)>,
    set_state: AtomicBool,
    transitioning: AtomicBool,
    deps: &'a [&'a Supply],
    name: &'static str,
}

/// Power supply controlled by a single "enable" line on GPIO
pub struct GpioSwitchedSupply<'a> {
    gpio: &'a Gpio,
    disch: Option<(&'a Gpio, u32)>,
    wait_ticks: u32,    // Number of 1ms ticks to wait after switching
                        // to consider the supply settled
    deps: &'a [&'a Supply],
    name: &'static str,
}

impl<'a> VrmSupply<'a> {
    pub const fn new(
        name: &'static str,
        deps: &'a [&'a Supply],
        vrm_id: u8,
        disch: Option<(&'a Gpio, u32)>,
    ) -> VrmSupply<'a>
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

    pub const CTRL_BIT_ENABLED: u8 = 1u8 << 0;
    pub const CTRL_BIT_POWER_GOOD: u8 = 1u8 << 1;
}

impl<'a> Supply for VrmSupply<'a> {
    fn status(&self) -> Result<SupplyStatus, Error>
    {
        let up_bits = VrmSupply::CTRL_BIT_ENABLED |
                      VrmSupply::CTRL_BIT_POWER_GOOD;
        let mut buf = [0u8; 1];
        VRM901.lock().read(&[self.vrm_id], &mut buf)?;
        let up = buf[0] & up_bits == up_bits;

        if self.transitioning.load(Ordering::Relaxed) {
            if up == self.set_state.load(Ordering::Relaxed) {
                self.transitioning.store(false, Ordering::Relaxed);
                if up {
                    Ok(SupplyStatus::Up)
                } else {
                    Ok(SupplyStatus::Down)
                }
            } else {
                Ok(SupplyStatus::Transition)
            }
        } else {
            if up {
                Ok(SupplyStatus::Up)
            } else {
                Ok(SupplyStatus::Down)
            }
        }
    }

    fn up(&self) -> StdResult
    {
        assert!(self.count_deps_not_up()? == 0);

        match self.disch {
            Some((gpio, _wait)) => {
                gpio.set(false);
            },
            None => (),
        }
        VRM901.lock().write(
            &[self.vrm_id],
            &[VrmSupply::CTRL_BIT_ENABLED],
        )?;
        self.set_state.store(true, Ordering::SeqCst);
        self.transitioning.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn down(&self) -> StdResult
    {
        assert!(self.count_rev_deps_not_down()? == 0);

        VRM901.lock().write(&[self.vrm_id], &[0])?;

        self.set_state.store(false, Ordering::Relaxed);
        self.transitioning.store(false, Ordering::Relaxed);

        if let Some((gpio, wait)) = self.disch {
            gpio.set(true);
            os::susp_safe_delay(wait);
        }

        Ok(())
    }

    fn deps(&self) -> &[&Supply]
    {
        &self.deps
    }

    fn name(&self) -> &'static str
    {
        &self.name
    }
}

impl<'a> GpioSwitchedSupply<'a> {
    pub const fn new(
        name: &'static str,
        deps: &'a [&'a Supply],
        gpio: &'a Gpio,
        wait_ticks: u32,
        disch: Option<(&'a Gpio, u32)>,
    ) -> GpioSwitchedSupply<'a>
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

impl<'a> Supply for GpioSwitchedSupply<'a> {
    fn status(&self) -> Result<SupplyStatus, Error>
    {
        if self.gpio.get() {
            Ok(SupplyStatus::Up)
        } else {
            Ok(SupplyStatus::Down)
        }
    }

    fn up(&self) -> StdResult
    {
        assert!(self.count_deps_not_up()? == 0);

        if let Some((disgpio, _wait)) = self.disch {
            disgpio.set(false);
        }
        self.gpio.set(true);
        os::susp_safe_delay(self.wait_ticks);
        Ok(())
    }

    fn down(&self) -> StdResult
    {
        assert!(self.count_rev_deps_not_down()? == 0);

        let mut max_wait = self.wait_ticks;

        self.gpio.set(false);
        if let Some((disgpio, wait)) = self.disch {
            disgpio.set(true);
            if wait > max_wait {
                max_wait = wait;
            }
        }
        os::susp_safe_delay(max_wait);
        Ok(())
    }

    fn deps(&self) -> &[&Supply]
    {
        &self.deps
    }

    fn name(&self) -> &'static str
    {
        &self.name
    }
}


unsafe impl<'a> Sync for VrmSupply<'a> {}
unsafe impl<'a> Sync for GpioSwitchedSupply<'a> {}
