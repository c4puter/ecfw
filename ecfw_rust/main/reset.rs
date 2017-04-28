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

use rustsys::ec_io;
use os;
use drivers;
use drivers::power::Supply;
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
        let _plock = drivers::power::POWER_MUTEX.lock_timeout(1000).expect("timeout");
        debug!(DEBUG_RESET, "acquire VRM TWI lock");
        let _lock = devices::twi::VRM901.lock_timeout(1000).expect("timeout");

        // Unsafe: shuts down the task scheduler
        debug!(DEBUG_RESET, "suspend tasks");
        ec_io::flush_output();
        unsafe {os::freertos::suspend_all()};
    }

    // Locks are released now; may be picked up again by the individual supply
    // control methods
    debug_async!(DEBUG_RESET, "shut down supplies");
    shutdown_supplies_cleanly();
    debug_async!(DEBUG_RESET, "shut down standby rail");
    unsafe{ shutdown_final(); }

    loop {}
}

/// Cleanly shut down all the power supplies. This ignores reference counting
/// and manually follows dependencies, in case the supply management code is
/// fucked up.
pub fn shutdown_supplies_cleanly()
{
    let _lock = drivers::power::POWER_MUTEX.lock();

    static SUPPLIES_IN_ORDER: &[&(devices::supplies::Supply + Sync)] = &[
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
            Err(e) => {println_async!("WARNING: {:?}", e);}
        }
        match supply.wait_status(drivers::power::SupplyStatus::Down) {
            Ok(_) => (),
            Err(e) => {println_async!("WARNING: {:?}", e);}
        }
    }
}

/// Shut down the standby rail, which powers the EC itself. The VRM will
/// bring it back up after a timeout.
unsafe fn shutdown_final()
{
    devices::supplies::BUCK_3VB.down().unwrap();
}
