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
use devices;
use drivers::gpio::Gpio;
use drivers::{ext4, gpt};
use devices::pins::*;
use devices::supplies::*;
use main::reset;
use messages::*;
use core::sync::atomic::*;

// Power button delays
const POWER_BUTTON_START_CYCLES_MAX: u32 = 5; // <1s: start
const POWER_BUTTON_STOP_CYCLES_MIN: u32 = 20; // >4s: stop

#[derive(Copy, Clone, Debug)]
pub enum Event {
    Boot,
    Shutdown,
    Reboot,
}

static POWER_STATE: AtomicUsize = ATOMIC_USIZE_INIT;
const STATE_RUN: usize = 0;
const STATE_SUSP: usize = 3;
const STATE_OFF: usize = 5;
const STATE_SHUTDOWN_FAIL: usize = 100;

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
            debug!(DEBUG_SYSMAN, "error on system event: {}", e);
        }
    }
}

fn handle_one_event(evt: Event) -> StdResult
{
    match evt {
        Event::Boot => do_safe_boot()?,
        Event::Shutdown => do_safe_shutdown()?,
        Event::Reboot => do_reboot()?,
    }

    Ok(())
}

/// Supply/LED status indication struct. This pairs a power supply with the LEDs
/// that indicate its status.
#[derive(Copy, Clone)]
struct SupplyStatusPair<'a> {
    supply: &'a (drivers::power::Supply + Sync),
    good: &'a drivers::ledmatrix::LedGpio<'a>,
    bad: &'a drivers::ledmatrix::LedGpio<'a>,
}

static SUPPLY_STATUS_TABLE: &[SupplyStatusPair] = &[
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

    POWER_STATE.store(STATE_OFF, Ordering::SeqCst);

    if FORCE_POWER.get() {
        post(Event::Boot);
    }

    loop {
        if cycle_count == 0 {
            let mut mat = devices::MATRIX.write();

            for &pair in SUPPLY_STATUS_TABLE {
                let stat = pair.supply.status();

                let (good, bad) = match stat {
                    Ok(SupplyStatus::Down)        => ( false, false ),
                    Ok(SupplyStatus::Up)          => ( true,  false ),
                    Ok(SupplyStatus::Transition)  => ( true,  true ),
                    Ok(SupplyStatus::Error)       => ( false, true ),
                    Err(_) => (false, true),
                };

                mat.buffer_led(pair.good.addr, good, false);
                mat.buffer_led(pair.bad.addr, bad, false);
            }

            mat.flush().unwrap();
        }

        // Handle power LED
        let state = POWER_STATE.load(Ordering::SeqCst);
        POWER_LED.set(state == STATE_RUN);

        // Handle power button
        if POWER_BTN.get() {
            if powerbtn_cycles_held == 0 {
                debug!(DEBUG_PWRBTN, "button pressed");
            }
            powerbtn_cycles_held += 1;
            if powerbtn_cycles_held >= POWER_BUTTON_STOP_CYCLES_MIN &&
               !powerbtn_handled {
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

        if state == STATE_RUN {
            debug!(DEBUG_SYSMAN, "TODO: power event to CPU");
        } else if state == STATE_SUSP {
            debug!(DEBUG_SYSMAN, "TODO: wake from S3");
        } else if state == STATE_OFF {
            post(Event::Boot);
        } else if state == STATE_SHUTDOWN_FAIL {
            debug!(
                DEBUG_SYSMAN,
                "ignoring power button because previous shutdown failed"
            );
            debug!(DEBUG_SYSMAN, "power cycle or use debug interface");
        }

    } else if cycles >= POWER_BUTTON_STOP_CYCLES_MIN {
        debug!(DEBUG_PWRBTN, "handle long press, state {}", state);

        if state == STATE_RUN {
            post(Event::Shutdown);
        } else if state == STATE_SUSP {
            post(Event::Shutdown);
        } else if state == STATE_SHUTDOWN_FAIL {
            debug!(
                DEBUG_SYSMAN,
                "ignoring power button because previous shutdown failed"
            );
            debug!(DEBUG_SYSMAN, "power cycle or use debug interface");
        }
    }
}

fn do_boot() -> StdResult
{
    debug!(DEBUG_SYSMAN, "boot");
    POWER_R.set(true);
    POWER_G.set(true);

    reset_fpgas();

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

    boot_init_clock()?;

    debug!(DEBUG_SYSMAN, "start USB-CDC");
    devices::COMCDC.start();

    POWER_R.set(false);
    devices::MATRIX.write().set_full_brightness()?;
    POWER_STATE.store(STATE_RUN, Ordering::SeqCst);

    if let Err(e) = boot_mount_card() {
        if e == ERR_NO_CARD {
            CARD_R.set_blink();
        } else {
            CARD_R.set(true);
        }
        CARD_G.set(false);
        return Err(e);
    } else {
        CARD_G.set(true);
        CARD_R.set(false);
    }

    boot_load_fpgas()?;

    unsafe {
        os::freertos::suspend_all();
    }
    SPEAKER.set(true);
    os::susp_safe_delay(125);
    SPEAKER.set(false);
    unsafe {
        os::freertos::resume_all();
    }
    Ok(())
}

fn boot_mount_card() -> StdResult
{
    if !CARD.get() {
        return Err(ERR_NO_CARD);
    }

    CARDEN.set(true);
    os::delay(1);
    devices::SD.lock().check()?;

    let mut table = gpt::Gpt::new(&devices::SD);
    let mut entry = gpt::GptEntry::new();

    table.read_header()?;
    table.read_boot(&mut entry)?;

    if !entry.valid() {
        return Err(ERR_NO_BOOT_PART);
    }

    let bd = ext4::makedev(&devices::SD, &entry);

    if let Err(e) = ext4::register_device(bd, "root") {
        if e == ERR_EEXIST {
            debug!(DEBUG_SYSMAN, "card already mounted, ignore mount failure");
            return Ok(());
        } else {
            return Err(e);
        }
    }

    ext4::mount("root", "/", false)?;

    Ok(())
}

fn boot_init_clock() -> StdResult
{
    debug!(DEBUG_SYSMAN, "initialize clock synthesizer");
    devices::CLOCK_SYNTH.y1div(25)?;
    devices::CLOCK_SYNTH.y2div(3)?;

    if LOW_SPEED.get() {
        debug!(DEBUG_SYSMAN, "LOW SPEED set");
        devices::CLOCK_SYNTH.y3div(20)?;
        debug!(
            DEBUG_SYSMAN,
            "EC ref: 7.5 MHz, bridge ref: 62.5 MHz, CPU: 9.375 MHz"
        );
    } else {
        devices::CLOCK_SYNTH.y3div(2)?;
        debug!(
            DEBUG_SYSMAN,
            "EC ref: 7.5 MHz, bridge ref: 62.5 MHz, CPU: 93.75 MHz"
        );
    }

    devices::CLOCK_SYNTH.ratio(75, 8)?;
    devices::CLOCK_SYNTH.usepll(true)?;
    unsafe {
        devices::CLOCK_SYNTH.enable_mck();
    }

    Ok(())
}

fn reset_fpgas()
{
    FPGA_PROG0.set(true);
    FPGA_PROG1.set(true);
    FPGA_PROG2.set(true);
    BIT_BRIDGE_G.set(false);
    BIT_CPU0_G.set(false);
    BIT_CPU1_G.set(false);
    BIT_R.set(false);
}

fn boot_load_fpgas() -> StdResult
{
    boot_load_fpga(0, "/bridge.bit", &BIT_BRIDGE_G)?;
    // boot_load_fpga(1, "/cpu0.bit", &BIT_CPU0_G)?;
    // boot_load_fpga(2, "/cpu1.bit", &BIT_CPU1_G)?;
    Ok(())
}

fn boot_load_fpga(n: usize, path: &str, led: &Gpio) -> StdResult
{
    debug!(DEBUG_SYSMAN, "load bitstream {} to FPGA {}", path, n);
    if let Err(e) = devices::FPGAS[n].load(path) {
        BIT_R.set(true);
        Err(e)
    } else {
        led.set(true);
        Ok(())
    }
}

fn do_shutdown() -> StdResult
{
    debug!(DEBUG_SYSMAN, "shutdown");
    POWER_R.set(true);

    reset_fpgas();

    if let Err(_) = ext4::umount("/") {
        debug!(DEBUG_SYSMAN, "card not mounted, ignore umount failure");
    } else {
        CARD_R.set(true);
        ext4::unregister_device("root")?;
        CARD_R.set(false);
        CARD_G.set(false);
    }

    debug!(DEBUG_SYSMAN, "stop USB-CDC");
    devices::COMCDC.stop();

    unsafe {
        devices::CLOCK_SYNTH.disable_mck();
    }

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
    POWER_STATE.store(STATE_OFF, Ordering::SeqCst);
    devices::MATRIX.write().set_standby_brightness()?;
    Ok(())
}

fn do_reboot() -> StdResult
{
    debug!(DEBUG_SYSMAN, "reboot");
    do_safe_shutdown()?;
    os::delay(750);
    do_safe_boot()?;
    Ok(())
}

fn do_safe_boot() -> StdResult
{
    if let Err(e) = do_boot() {
        STATE_FAIL_R.set(true);
        if let Err(e2) = recover_boot() {
            STATE_FAIL_R.set_blink();
            panic!("error recovering from failed state change: {}", e2);
        }
        POWER_STATE.store(STATE_OFF, Ordering::SeqCst);
        Err(e)
    } else {
        STATE_FAIL_R.set(false);
        Ok(())
    }
}

fn do_safe_shutdown() -> StdResult
{
    if let Err(e) = do_shutdown() {
        STATE_FAIL_R.set(true);
        if let Err(e2) = recover_shutdown() {
            STATE_FAIL_R.set_blink();
            panic!("error recovering from failed state change: {}", e2);
        }
        POWER_STATE.store(STATE_SHUTDOWN_FAIL, Ordering::SeqCst);
        Err(e)
    } else {
        STATE_FAIL_R.set(false);
        Ok(())
    }
}

fn recover_boot() -> StdResult
{
    // Instead of tearing down in reverse of startup order, tear down in the
    // order that gets us into the safest state possible if teardown fails

    debug!(DEBUG_SYSMAN, "failed to boot, recovering");
    debug!(DEBUG_SYSMAN, "stop USB-CDC");
    devices::COMCDC.stop();

    debug!(DEBUG_SYSMAN, "quick supply shutdown");
    POWER_R.set(true);
    POWER_G.set(false);
    reset::shutdown_supplies_cleanly();
    debug!(DEBUG_SYSMAN, "reached S5");
    POWER_R.set(false);

    if let Err(_) = ext4::umount("/") {
        debug!(DEBUG_SYSMAN, "card not mounted, ignore umount failure");
    } else {
        CARD_R.set(true);
        ext4::unregister_device("root")?;
        CARDEN.set(false);
        CARD_R.set(false);
        CARD_G.set(false);
    }

    devices::MATRIX.write().set_standby_brightness()?;

    Ok(())
}

fn recover_shutdown() -> StdResult
{
    // Actually the same thing as recovering from a failed boot, as in both
    // cases we just want to reach "OFF" state as directly as possible.
    recover_boot()
}
