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

use os;
use drivers;
use drivers::gpio::Gpio;
use devices::pins::*;
use devices::supplies::*;
use messages::*;
use core::sync::atomic::*;

// Power button delays
const POWER_BUTTON_START_CYCLES_MAX: u32 = 5; // <1s: start
const POWER_BUTTON_STOP_CYCLES_MIN: u32 = 20; // >4s: stop

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
    debug!(DEBUG_SYSMAN, "start");

    loop {
        let event = EVENTS.receive_wait_blocking();
        if let Err(e) = handle_one_event(event) {
            panic!("Error on system event: {}", e);
        }
    }
}

fn handle_one_event(evt: Event) -> StdResult
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
    supply: &'static(drivers::power::Supply + Sync),
    good: &'static drivers::ledmatrix::LedGpio,
    bad: &'static drivers::ledmatrix::LedGpio,
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
    let mut powerbtn_cycles_held = 0u32;
    let mut powerbtn_handled = false;
    let mut lastwake = os::ticks_running();
    let mut cycle_count = 0;

    POWER_STATE.store(5, Ordering::SeqCst);

    if FORCE_POWER.get() {
        post(Event::Boot);
    }

    loop {
        if cycle_count == 0 {
            let mut mat = drivers::ledmatrix::MATRIX.write();

            for &pair in SUPPLY_STATUS_TABLE {
                let stat = pair.supply.status().unwrap();

                let (good, bad) = match stat {
                    SupplyStatus::Down        => ( false, false ),
                    SupplyStatus::Up          => ( true,  false ),
                    SupplyStatus::Transition  => ( true,  true ),
                    SupplyStatus::Error       => ( false, true ),
                };

                mat.buffer_led(pair.good.addr, good);
                mat.buffer_led(pair.bad.addr, bad);
            }

            mat.flush().unwrap();
        }

        // Handle power LED
        let state = POWER_STATE.load(Ordering::SeqCst);
        POWER_LED.set(state == 0);

        // Handle power button
        if POWER_BTN.get() {
            if powerbtn_cycles_held == 0 {
                debug!(DEBUG_PWRBTN, "button pressed");
            }
            powerbtn_cycles_held += 1;
            if powerbtn_cycles_held >= POWER_BUTTON_STOP_CYCLES_MIN && !powerbtn_handled {
                debug!(DEBUG_PWRBTN, "press event (long)");
                powerbtn_handled = true;
                button_press(powerbtn_cycles_held);
            }
        } else if powerbtn_cycles_held > 0 {
            debug!(DEBUG_PWRBTN, "button released");
            if !powerbtn_handled {
                debug!(DEBUG_PWRBTN, "press event");
                button_press(powerbtn_cycles_held);
            }
            powerbtn_handled = false;
            powerbtn_cycles_held = 0;
        }

        os::delay_period(&mut lastwake, 100);
        cycle_count = (cycle_count + 1) % 6;
    }
}

fn button_press(cycles: u32)
{
    let state = POWER_STATE.load(Ordering::SeqCst);

    if cycles <= POWER_BUTTON_START_CYCLES_MAX {
        debug!(DEBUG_PWRBTN, "handle short press, state {}", state);

        if state == 0 {
            debug!(DEBUG_SYSMAN, "TODO: power event to CPU");
        } else if state == 3 {
            debug!(DEBUG_SYSMAN, "TODO: wake from S3");
        } else if state == 5 {
            post(Event::Boot);
        }

    } else if cycles >= POWER_BUTTON_STOP_CYCLES_MIN {
        debug!(DEBUG_PWRBTN, "handle long press, state {}", state);

        if state == 0 {
            post(Event::Shutdown);
        } else if state == 3 {
            post(Event::Shutdown);
        }

    }
}

fn do_boot() -> StdResult
{
    debug!(DEBUG_SYSMAN, "boot");
    POWER_R.set(true);
    POWER_G.set(true);

    if let Err(e) = transition_s3_from_s5() {
        POWER_G.set(false);
        return Err(e);
    } else {
        debug!(DEBUG_SYSMAN, "reached S3");
    }

    if let Err(e) = transition_s0_from_s3() {
        POWER_G.set(false);
        return Err(e);
    } else {
        debug!(DEBUG_SYSMAN, "reached S0");
    }

    POWER_R.set(false);
    try!(drivers::ledmatrix::MATRIX.write().set_brightness(drivers::ledmatrix::FULL_BRIGHTNESS));
    POWER_STATE.store(0, Ordering::SeqCst);
    SPEAKER.set(true);
    os::delay(125);
    SPEAKER.set(false);
    Ok(())
}

fn do_shutdown() -> StdResult
{
    debug!(DEBUG_SYSMAN, "shutdown");
    POWER_R.set(true);

    if let Err(e) = transition_s3_from_s0() {
        POWER_G.set(false);
        return Err(e);
    } else {
        debug!(DEBUG_SYSMAN, "reached S3");
    }

    if let Err(e) = transition_s5_from_s3() {
        POWER_G.set(false);
        return Err(e);
    } else {
        debug!(DEBUG_SYSMAN, "reached S5");
    }

    POWER_R.set(false);
    POWER_G.set(false);
    POWER_STATE.store(5, Ordering::SeqCst);
    try!(drivers::ledmatrix::MATRIX.write().set_brightness(drivers::ledmatrix::STANDBY_BRIGHTNESS));
    Ok(())
}

fn do_reboot() -> StdResult
{
    debug!(DEBUG_SYSMAN, "reboot");
    try!(do_shutdown());
    os::delay(750);
    try!(do_boot());
    Ok(())
}
