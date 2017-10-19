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
use drivers;
use drivers::power::Supply;
use drivers::com::Com;
use devices;
extern crate asf_rstc;

/// Reset the system by shutting down all the power supplies including the
/// standby rail. The VRM will automatically bring the standby rail back up
/// after a timeout.
pub fn hard_reset()
{
    // Grab locks before suspending tasks. This allows any pending
    // transactions to complete, avoiding putting the VRM's I2C slave
    // in an unknown state.
    {
        debug!(DEBUG_RESET, "acquire power control lock");
        let _plock = drivers::power::POWER_MUTEX.lock_timeout(1000).expect(
            "timeout",
        );
        debug!(DEBUG_RESET, "acquire VRM TWI lock");
        let _lock = devices::twi::VRM901.lock_timeout(1000).expect("timeout");

        // Unsafe: shuts down the task scheduler
        debug!(DEBUG_RESET, "suspend tasks");
        devices::COMUSART.flush_output();
        unsafe { os::freertos::suspend_all() };
    }

    // Locks are released now; may be picked up again by the individual supply
    // control methods
    debug_async!(DEBUG_RESET, "shut down supplies");
    shutdown_supplies_cleanly();
    debug_async!(DEBUG_RESET, "shut down standby rail");
    unsafe {
        shutdown_final();
    }

    loop {}
}

/// Cleanly shut down all the power supplies. This ignores reference counting
/// and manually follows dependencies, in case the supply management code is
/// fucked up.
pub fn shutdown_supplies_cleanly()
{
    let _lock = drivers::power::POWER_MUTEX.lock();

    unsafe {
        devices::CLOCK_SYNTH.disable_mck();
    }

    static SUPPLIES_IN_ORDER: &[&(devices::supplies::Supply + Sync)] =
        &[
            &devices::supplies::SW1,
            &devices::supplies::SW2,
            &devices::supplies::SW3,
            &devices::supplies::LDO_S0,
            &devices::supplies::LDO_S3,
            &devices::supplies::BUCK_1V2,
            &devices::supplies::BUCK_1V5,
            &devices::supplies::INV_N12,
            &devices::supplies::BUCK_5VA,
            &devices::supplies::BUCK_5VB,
            &devices::supplies::BUCK_3VA,
        ];
    for supply in SUPPLIES_IN_ORDER {
        match supply.down() {
            Ok(_) => (),
            Err(e) => {
                print_async!("WARNING: {:?}\n", e);
            },
        }
        match supply.wait_status(drivers::power::SupplyStatus::Down) {
            Ok(_) => (),
            Err(e) => {
                print_async!("WARNING: {:?}\n", e);
            },
        }
    }
}

/// Shut down the standby rail, which powers the EC itself. The VRM will
/// bring it back up after a timeout.
unsafe fn shutdown_final()
{
    devices::supplies::BUCK_3VB.down().unwrap();
}
