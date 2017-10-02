/*
 * c4puter embedded controller firmware
 * Copyright (C) 2017 Chris Pavlina
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

use os::Mutex;
use messages::*;
extern crate bindgen_mcu;
use core::sync::atomic::*;

pub struct Spi {
    mutex: Mutex<()>,
    initialized: AtomicBool,
}

/// Threadsafe wrapper around SPI peripheral. This must be iniitalized before
/// use; use before init() or a double-init() will panic.
impl Spi {
    pub const fn new() -> Spi {
        Spi {
            mutex: Mutex::new(()),
            initialized: ATOMIC_BOOL_INIT,
        }
    }

    pub fn init(&self) -> StdResult {
        let was_initialized = self.initialized.swap(true, Ordering::Relaxed);
        if was_initialized {
            panic!("SPI: double init()");
        }
        unsafe {
            bindgen_mcu::mcu_init_spi();
        }
        Ok(())
    }

    pub fn write(&self, buffer: &[u8]) -> StdResult {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("SPI: use before init()");
        }
        let _lock = self.mutex.lock();

        for b in buffer.iter() {
            let failed = unsafe {
                bindgen_mcu::mcu_spi_write(*b)
            };
            if failed {
                return Err(ERR_TIMEOUT);
            }
        }
        Ok(())
    }
}
