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

//! Driver to load bitstreams onto FPGAs.

use os::Mutex;
use os::freertos;
use messages::*;
use drivers::spi::Spi;
use drivers::gpio::Gpio;
use drivers::ext4;
use alloc::vec;

/// Driver for programming a Spartan 6.
pub struct Spartan6<'a> {
    mutex: &'a Mutex<()>,
    spi: &'a Spi,
    done_pin: &'a (Gpio + Sync),
    init_pin: &'a (Gpio + Sync),
    prog_pin: &'a (Gpio + Sync),
}

impl<'a> Spartan6<'a> {
    /// Construct a Spartan6 device.
    ///
    /// # Arguments
    ///
    /// * `mutex`    - Mutex to lock the programming interface. If any
    ///                hardware (SPI) is shared between FPGAs, they must
    ///                share a mutex.
    /// * `spi`      - SPI peripheral for sending code to the FPGA.
    /// * `done_pin` - Input to indicate when the FPGA is done
    ///                initializing. This may not be shared between
    ///                FPGAs.
    /// * `init_pin` - IO to indicate initialization status. This should
    ///                not be shared between FPGAs, and should be
    ///                inverted to reflect the active-low hardware
    ///                interface.
    /// * `prog_pin` - Output to put the FPGA in programming mode. This
    ///                may not be shared between FPGAs, and should be
    ///                inverted to reflect the active-low hardware
    ///                interface.
    pub const fn new<'b>(
        mutex: &'b Mutex<()>,
        spi: &'b Spi,
        done_pin: &'b (Gpio + Sync),
        init_pin: &'b (Gpio + Sync),
        prog_pin: &'b (Gpio + Sync),
    ) -> Spartan6<'b>
    {
        Spartan6 {
            mutex: mutex,
            spi: spi,
            done_pin: done_pin,
            init_pin: init_pin,
            prog_pin: prog_pin,
        }
    }

    /// Initialize the FPGA with the given filename
    pub fn load(&self, filename: &str) -> StdResult
    {
        let _lock = self.mutex.lock();

        self.prog_pin.set(true);
        freertos::delay(1);
        self.prog_pin.set(false);
        wait_for_pin(self.init_pin, false, 100)?;

        self.actual_load(filename)?;

        wait_for_pin(self.done_pin, true, 500)?;

        Ok(())
    }

    /// The actual file transfer without the pin twiddling
    fn actual_load(&self, filename: &str) -> StdResult
    {
        let mut file = ext4::fopen_expand(filename, ext4::OpenFlags::Read)?;

        let mut buf1 = vec::from_elem(0u8, 4096);
        let mut buf2 = vec::from_elem(0u8, 4096);

        let mut wr1 = None;

        loop {
            let n_read1 = file.read(&mut buf1)?;

            if let Some(wr) = wr1.take() {
                self.spi.end_write(wr);
            }

            if n_read1 == 0 {
                break;
            }

            let wr2 = self.spi.start_write(&buf1[0 .. n_read1])?;

            let n_read2 = file.read(&mut buf2)?;
            self.spi.end_write(wr2);

            if n_read2 == 0 {
                break;
            }

            wr1 = Some(self.spi.start_write(&buf2[0 .. n_read2])?);
        }

        Ok(())
    }
}

fn wait_for_pin(pin: &Gpio, state: bool, timeout_ticks: u32) -> StdResult
{
    let end_tick = freertos::ticks().wrapping_add(timeout_ticks);

    while end_tick < freertos::ticks() {
        if pin.get() == state {
            return Ok(());
        }
        freertos::yield_task();
    }

    while freertos::ticks() < end_tick {
        if pin.get() == state {
            return Ok(());
        }
        freertos::yield_task();
    }

    Err(ERR_TIMEOUT)
}
