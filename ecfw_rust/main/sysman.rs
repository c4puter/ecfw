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
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
 * OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
 * OR OTHER DEALINGS IN THE SOFTWARE.
 */

use rustsys::{queue, ec_io, freertos};
use main::power;
use main::supplies::*;
use main::pins::*;
use hardware::gpio::*;
use core::sync::atomic::*;

#[derive(Copy,Clone,Debug)]
pub enum Event {
    Boot,
    Shutdown,
    Reboot,
}

static POWER_STATE: AtomicUsize = ATOMIC_USIZE_INIT;

queue_static_new!(EVENTS: [Event; 2]);

/// Post an event. Returns immediately after the event is added to the queue.
pub fn post(event: Event)
{
    EVENTS.send_wait(event);
}

/// Event loop task
pub fn run_event()
{
    EVENTS.register_receiver();
    println!("Start system manager");

    loop {
        let event = EVENTS.receive_wait_blocking();
        if let Err(e) = handle_one_event(event) {
            panic!("Error on system event: {}", e);
        }
    }
}

fn handle_one_event(evt: Event) -> Result<(),&'static str>
{
    try!(match evt {
        Event::Boot => do_boot(),
        Event::Shutdown => do_shutdown(),
        Event::Reboot => do_reboot(),
    });

    Ok(())
}

/// Supply/LED status indication struct. This pairs a power supply with the LEDs
/// that indicate its status.
#[derive(Copy,Clone)]
struct SupplyStatusPair {
    supply: &'static(power::Supply + Sync),
    good: &'static(Gpio + Sync),
    bad: &'static(Gpio + Sync),
}

static SUPPLY_STATUS_TABLE: &'static [SupplyStatusPair] = &[
    SupplyStatusPair{ supply: &SW1,              good: &P12V_PCI_G,   bad: &P12V_PCI_R },
    SupplyStatusPair{ supply: &BUCK_5VA,         good: &P5V_PCI_A_G,  bad: &P5V_PCI_A_R },
    SupplyStatusPair{ supply: &SW2,              good: &P5V_PCI_B_G,  bad: &P5V_PCI_B_R },
    SupplyStatusPair{ supply: &BUCK_3VA,         good: &P3V3_PCI_A_G, bad: &P3V3_PCI_A_R },
    SupplyStatusPair{ supply: &SW3,              good: &P3V3_PCI_B_G, bad: &P3V3_PCI_B_R },
    SupplyStatusPair{ supply: &INV_N12,          good: &N12V_PCI_G,   bad: &N12V_PCI_R },
    SupplyStatusPair{ supply: &BUCK_3VB,         good: &P3V3_STBY_G,  bad: &P3V3_STBY_R },
    SupplyStatusPair{ supply: &BUCK_3VA,         good: &P3V3_AUX_G,   bad: &P3V3_AUX_R },
    SupplyStatusPair{ supply: &SW3,              good: &P3V3_LOGIC_G, bad: &P3V3_LOGIC_R },
    SupplyStatusPair{ supply: &BUCK_1V5,         good: &P1V5_LOGIC_G, bad: &P1V5_LOGIC_R },
    SupplyStatusPair{ supply: &BUCK_1V2,         good: &P1V2_LOGIC_G, bad: &P1V2_LOGIC_R },
    SupplyStatusPair{ supply: &LDO_S0,           good: &PV75_TERM_G,  bad: &PV75_TERM_R },
];

/// Status manager task
pub fn run_status()
{
    let mut powerbtn_cycles_held = 0;
    let mut lastwake = freertos::ticks_running();

    POWER_STATE.store(5, Ordering::SeqCst);

    loop {
        for &pair in SUPPLY_STATUS_TABLE {
            let stat = pair.supply.status().unwrap();

            match stat {
                SupplyStatus::Down        => { pair.good.set(false); pair.bad.set(false); },
                SupplyStatus::Up          => { pair.good.set(true);  pair.bad.set(false); },
                SupplyStatus::Transition  => { pair.good.set(true);  pair.bad.set(true); },
                SupplyStatus::Error       => { pair.good.set(false); pair.bad.set(true); },
            }

        }

        // Handle power LED
        let state = POWER_STATE.load(Ordering::SeqCst);
        POWER_LED.set(state == 0);

        // Handle power button
        if POWER_BTN.get() {
            powerbtn_cycles_held += 1;
        } else if powerbtn_cycles_held > 0 {
            button_press(powerbtn_cycles_held);
            powerbtn_cycles_held = 0;
        }

        freertos::delay_period(&mut lastwake, 200);
    }
}

fn button_press(cycles: u32)
{
    let state = POWER_STATE.load(Ordering::SeqCst);

    if state == 0 {
        if cycles <= 5 {
            // Less than 1 second: send power event to CPU
            println!("TODO: power event to CPU");
        } else if cycles >= 20 {
            // More than 4 seconds: force shutdown
            post(Event::Shutdown);
        }
    } else if state == 3 {
        if cycles <= 5 {
            // Less than 1 second in S3: wake up
            println!("TODO: wake from S3");
        } else if cycles >= 20 {
            // More than 4 seconds: force shutdown
            post(Event::Shutdown);
        }
    } else if state == 5 {
        if cycles <= 5 {
            post(Event::Boot);
        }
    } else {
        panic!("unhandled power state {}", state);
    }
}

fn do_boot() -> Result<(),&'static str>
{
    println!("sysman: boot");
    try!(transition_s3_from_s5());
    println!("sysman: reached S3");
    try!(transition_s0_from_s3());
    println!("sysman: reached S0");

    //SPEAKER.set(true);
    freertos::delay(250);
    SPEAKER.set(false);
    POWER_STATE.store(0, Ordering::SeqCst);
    Ok(())
}

fn do_shutdown() -> Result<(),&'static str>
{
    println!("sysman: shutdown");
    try!(transition_s3_from_s0());
    println!("sysman: reached S3");
    try!(transition_s5_from_s3());
    println!("sysman: reached S5");
    POWER_STATE.store(5, Ordering::SeqCst);
    Ok(())
}

fn do_reboot() -> Result<(),&'static str>
{
    println!("sysman: reboot");
    try!(do_shutdown());
    freertos::delay(750);
    try!(do_boot());
    Ok(())
}
