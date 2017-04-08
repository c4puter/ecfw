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

use rustsys::{ec_io,freertos};
use main::{power,supplies,twi_devices};
use main::power::Supply;
extern crate asf_rstc;

/// Reset the system by shutting down all the power supplies including the
/// standby rail. The VRM will automatically bring the standby rail back up
/// after a timeout.
pub fn hard_reset()
{
    // Grab locks before suspending tasks. This allows any pending
    // transactions to complete, avoiding putting the VRM's I2C slave
    // in an unknown state.
    println!("\nHard reset");
    println!("    Acquire power control lock");
    let _plock = power::POWER_MUTEX.lock_timeout(1000).expect("timeout");
    println!("    Acquire VRM I2C lock");
    let _lock = twi_devices::VRM901.lock_timeout(1000).expect("timeout");

    // Unsafe: shuts down the task scheduler
    println!("    Suspend tasks");
    ec_io::flush_output();
    unsafe {freertos::suspend_all()};

    println_async!("    Shut down supplies");
    shutdown_supplies_cleanly();
    println_async!("    Shut down standby rail");
    shutdown_final();

    loop {}
}

/// Cleanly shut down all the power supplies. This ignores reference counting
/// and manually follows dependencies, in case the supply management code is
/// fucked up.
fn shutdown_supplies_cleanly()
{
    static SUPPLIES_IN_ORDER: &'static [&'static(Supply + Sync)] = &[
        &supplies::SW1,
        &supplies::SW2,
        &supplies::SW3,
        &supplies::LDO_S0,
        &supplies::LDO_S3,
        &supplies::BUCK_1V2,
        &supplies::BUCK_1V5,
        &supplies::INV_N12,
        &supplies::BUCK_5VA,
        &supplies::BUCK_5VB,
        &supplies::BUCK_3VA,
    ];
    for supply in SUPPLIES_IN_ORDER {
        match supply.down() {
            Ok(_) => (),
            Err(e) => {println_async!("WARNING: {:?}", e);}
        }
        match supply.wait_status(power::SupplyStatus::Down) {
            Ok(_) => (),
            Err(e) => {println_async!("WARNING: {:?}", e);}
        }
    }
}

/// Shut down the standby rail, which powers the EC itself. The VRM will
/// bring it back up after a timeout.
fn shutdown_final()
{
    supplies::BUCK_3VB.down().unwrap();
}
