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

//! On-chip SPI driver (wrapper around `mcu.c` functions).

use os::{Mutex, MutexLock};
use messages::*;
extern crate bindgen_mcu;
extern crate asf_pdc;
extern crate ctypes;
use core::sync::atomic::*;
use core::ptr;

pub struct Spi {
    mutex: Mutex<()>,
    initialized: AtomicBool,
}

pub struct SpiDmaWrite<'a> {
    lock: Option<MutexLock<'a, ()>>,
}

/// Threadsafe wrapper around SPI peripheral. This must be iniitalized before
/// use; use before init() or a double-init() will panic.
impl Spi {
    pub const fn new() -> Spi
    {
        Spi {
            mutex: Mutex::new(()),
            initialized: ATOMIC_BOOL_INIT,
        }
    }

    pub fn init(&self) -> StdResult
    {
        let was_initialized = self.initialized.swap(true, Ordering::Relaxed);
        if was_initialized {
            panic!("SPI: double init()");
        }
        unsafe {
            bindgen_mcu::mcu_init_spi();
        }
        Ok(())
    }

    /// Start a write via DMA. end_write() must be called after this.
    pub fn start_write(&self, buffer: &[u8]) -> Result<SpiDmaWrite, Error>
    {
        if !self.initialized.load(Ordering::Relaxed) {
            panic!("SPI: use before init()");
        }

        let lock = self.mutex.lock();

        let mut packet = asf_pdc::pdc_packet {
            ul_addr: buffer.as_ptr() as u32,
            ul_size: buffer.len() as u32,
        };
        let pdc_base = unsafe { bindgen_mcu::mcu_spi_pdc_base() } as
                       *mut asf_pdc::Pdc;

        unsafe {
            asf_pdc::pdc_tx_init(pdc_base, &mut packet, ptr::null_mut());
            asf_pdc::pdc_enable_transfer(pdc_base, 0x00000100);
        }
        Ok(SpiDmaWrite { lock: Some(lock) })
    }

    /// Check whether a write has completed.
    pub fn write_finished(&self, _dmawrite: &SpiDmaWrite) -> bool
    {
        let pdc_base = unsafe { bindgen_mcu::mcu_spi_pdc_base() } as
                       *mut asf_pdc::Pdc;
        unsafe { asf_pdc::pdc_read_tx_counter(pdc_base) == 0 }
    }

    /// Clean up after a DMA write.
    pub fn end_write(&self, mut dmawrite: SpiDmaWrite)
    {
        while !self.write_finished(&dmawrite) {}
        let pdc_base = unsafe { bindgen_mcu::mcu_spi_pdc_base() } as
                       *mut asf_pdc::Pdc;
        unsafe {
            asf_pdc::pdc_disable_transfer(pdc_base, 0x00000200);
        }
        dmawrite.lock = None;
    }
}

impl<'a> Drop for SpiDmaWrite<'a> {
    fn drop(&mut self)
    {
        if self.lock.is_some() {
            panic!(
                "SPI DMA lock went out of scope without calling end_write()"
            );
        }
    }
}
