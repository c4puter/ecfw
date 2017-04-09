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
use hardware::twi::TWI0;
use hardware::gpio::*;
use hardware::tempsensor;
use hardware::sd::*;
use main::{pins, supplies, reset, sysman, debug};
use main::pins::*;

use main::parseint::ParseInt;
use core::fmt;

pub struct Command {
    pub name: &'static str,
    pub f: fn(args: &[&str]) -> Result<(), &'static str>,
    pub descr: &'static str,
}

pub static COMMAND_TABLE: &'static [Command] = &[
    Command{ name: "help",      f: cmd_help,    descr: "display commands and their descriptions" },
    Command{ name: "free",      f: cmd_free,    descr: "display free heap" },
    Command{ name: "reset",     f: cmd_reset,   descr: "reset entire system" },
    Command{ name: "dbgen",     f: cmd_dbgen,   descr: "enabled debug item" },
    Command{ name: "dbgdis",    f: cmd_dbgdis,  descr: "disable debug item" },
    Command{ name: "dbgls",     f: cmd_dbgls,   descr: "list debug items" },

    Command{ name: "panel",     f: cmd_panel,   descr: "render the user IO panel to the console" },
    Command{ name: "temps",     f: cmd_temps,   descr: "read the temperature sensors" },
    Command{ name: "event",     f: cmd_event,   descr: "send an event (boot, shutdown, reboot)" },

    Command{ name: "i2c_probe", f: cmd_i2c_probe,   descr: "probe I2C for an ADDR" },
    Command{ name: "i2c_read",  f: cmd_i2c_read,    descr: "read I2C from ADDR at LOCATION, N bytes" },
    Command{ name: "i2c_write", f: cmd_i2c_write,   descr: "write I2C to ADDR at LOCATION, BYTES" },

    Command{ name: "gpio_read", f: cmd_gpio_read,   descr: "read GPIO (by name)" },
    Command{ name: "gpio_write",f: cmd_gpio_write,  descr: "write to GPIO (by name) VALUE" },

    Command{ name: "pwr_stat",  f: cmd_pwr_stat,    descr: "display status of SUPPLY" },

    Command{ name: "mount",     f: cmd_mount,       descr: "mount SD card" },
    Command{ name: "umount",    f: cmd_umount,      descr: "unmount SD card" },
    Command{ name: "sdinfo",    f: cmd_sdinfo,      descr: "print SD card info" },
];

fn argv_parsed<T, U>(args: &[&str], n: usize, _name: &str, parser: fn(&str)->Result<T,U>) -> Result<T, &'static str>
    where U: fmt::Display
{
    match parser(args[n]) {
        Ok(val) => Ok(val),
        Err(_) => Err("cannot parse argument")
    }
}

fn cmd_help(_args: &[&str]) -> Result<(), &'static str>
{
    for i in 0..COMMAND_TABLE.len() {
        let ref cmd = COMMAND_TABLE[i];
        println!("{:12} - {}", cmd.name, cmd.descr);
    }

    Ok(())
}

fn cmd_free(_args: &[&str]) -> Result<(), &'static str>
{
    println!("{} B", freertos::get_free_heap());
    Ok(())
}

fn cmd_reset(_args: &[&str]) -> Result<(), &'static str>
{
    reset::hard_reset();
    Err("did not reset")    // should never happen
}

fn cmd_dbgen(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        Err("no debug item specified")
    } else {
        if debug::debug_set(args[1], true) {
            Ok(())
        } else {
            Err("cannot find debug item")
        }
    }
}

fn cmd_dbgdis(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        Err("no debug item specified")
    } else {
        if debug::debug_set(args[1], false) {
            Ok(())
        } else {
            Err("cannot find debug item")
        }
    }
}

fn cmd_dbgls(_args: &[&str]) -> Result<(), &'static str>
{
    for &dbg in debug::DEBUG_TABLE {
        println!("{}    {}",
            if dbg.enabled() { "en " } else { "dis" },
            dbg.name );
    }
    Ok(())
}

fn cmd_panel(_args: &[&str]) -> Result<(), &'static str>
{
    fn r_(v: bool) -> &'static str {
        match v { true => "R ", false => "  " }
    }
    fn g_(v: bool) -> &'static str {
        match v { true => "G ", false => "  " }
    }
    fn yn(v: bool) -> &'static str {
        match v { true => "Y ", false => " N" }
    }

    println!("P12V   {} {} | P3V3_STBY  {} {} | EC FMW {} {}       {} {} UNC0",
             r_(P12V_PCI_R.get()), g_(P12V_PCI_G.get()),
             r_(P3V3_STBY_R.get()), g_(P3V3_STBY_G.get()),
             r_(ECFW_R.get()), g_(ECFW_G.get()),
             r_(UNC0_R.get()), g_(UNC0_G.get()));
    println!("P5V_A  {} {} | P3V3_AUX   {} {} | PWR SQ {} {}       {} {} UNC1",
             r_(P5V_PCI_A_R.get()), g_(P5V_PCI_A_G.get()),
             r_(P3V3_AUX_R.get()), g_(P3V3_AUX_G.get()),
             r_(POWER_R.get()), g_(POWER_G.get()),
             r_(UNC1_R.get()), g_(UNC1_G.get()));
    println!("P5V_B  {} {} | P3V3_LOGIC {} {} | CARD   {} {}       {} {} UNC2",
             r_(P5V_PCI_B_R.get()), g_(P5V_PCI_B_G.get()),
             r_(P3V3_LOGIC_R.get()), g_(P3V3_LOGIC_G.get()),
             r_(CARD_R.get()), g_(CARD_G.get()),
             r_(UNC2_R.get()), g_(UNC2_G.get()));
    println!("P3V3_A {} {} | P1V5_LOGIC {} {} | BITSTR {} {} {} {} {} {} UNC3",
             r_(P3V3_PCI_A_R.get()), g_(P3V3_PCI_A_G.get()),
             r_(P1V5_LOGIC_R.get()), g_(P1V5_LOGIC_G.get()),
             r_(BIT_R.get()), g_(BIT_BRIDGE_G.get()), g_(BIT_CPU0_G.get()), g_(BIT_CPU1_G.get()),
             r_(UNC3_R.get()), g_(UNC3_G.get()));
    println!("P3V3_B {} {} | P1V2_LOGIC {} {} | MEM LD {} {}       {} {} UNC4",
             r_(P3V3_PCI_B_R.get()), g_(P3V3_PCI_B_G.get()),
             r_(P1V2_LOGIC_R.get()), g_(P1V2_LOGIC_G.get()),
             r_(MEM_R.get()), g_(MEM_G.get()),
             r_(UNC4_R.get()), g_(UNC4_G.get()));
    println!("N12V   {} {} | PV75_TERM  {} {} | RUN    {} {}    {} {} {} UNC5",
             r_(N12V_PCI_R.get()), g_(N12V_PCI_G.get()),
             r_(PV75_TERM_R.get()), g_(PV75_TERM_G.get()),
             r_(RUN_R.get()), g_(RUN_G.get()), g_(UPDOG_G.get()),
             r_(UNC5_R.get()), g_(UNC5_G.get()));
    println!("");
    println!("{} UNC0", yn(UNC_SW_0.get()));
    println!("{} UNC1", yn(UNC_SW_1.get()));
    println!("{} UNC2", yn(UNC_SW_2.get()));
    println!("{} low speed", yn(LOW_SPEED.get()));
    println!("{} force pwr", yn(FORCE_POWER.get()));
    println!("{} single CPU", yn(SINGLE_CPU.get()));
    println!("{} debug boot", yn(DEBUG_BOOT.get()));
    println!("{} merged ser", yn(MERGE_SERIAL.get()));
    Ok(())
}

fn cmd_temps(_args: &[&str]) -> Result<(), &'static str>
{
    let temp_logic = try!(tempsensor::SENSOR_LOGIC.read());
    let temp_ambient = try!(tempsensor::SENSOR_AMBIENT.read());

    println!("Logic:   {}.{} degC", temp_logic/10, temp_logic%10);
    println!("Ambient: {}.{} degC", temp_ambient/10, temp_ambient%10);

    Ok(())
}

fn cmd_event(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        Err("no event specified")
    } else if args[1] == "boot" {
        sysman::post(sysman::Event::Boot);
        Ok(())
    } else if args[1] == "shutdown" {
        sysman::post(sysman::Event::Shutdown);
        Ok(())
    } else if args[1] == "reboot" {
        sysman::post(sysman::Event::Reboot);
        Ok(())
    } else {
        Err("unrecognized event name")
    }
}

fn cmd_i2c_probe(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        return Err("expected argument(s)");
    }
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));
    match TWI0.probe(addr) {
        Ok(is_present) => {
            if is_present { println!("address {} present", addr); }
            else          { println!("address {} does not respond", addr); }
            Ok(())
        },
        Err(e) => { Err(e.description()) }
    }
}

fn cmd_i2c_read(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 4 {
        return Err("expected argument(s)");
    }
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));
    let loc = try!(argv_parsed(args, 2, "LOCATION", u8::parseint));
    let n = try!(argv_parsed(args, 3, "N", u8::parseint));
    if n > 16 {
        return Err("can only read up to 16 bytes");
    }

    let location_arr = [loc];
    let mut buffer = [0 as u8; 16];

    match TWI0.read(addr, &location_arr, &mut buffer[0..n as usize]) {
        Ok(_) => {
            println!("{:?}", &buffer[0..n as usize]);
            Ok(())
        }
        Err(e) => Err(e.description())
    }
}

fn cmd_i2c_write(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 3 {
        return Err("expected argument(s)");
    }
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));
    let loc = try!(argv_parsed(args, 2, "LOCATION", u8::parseint));

    if args.len() > 19 {
        return Err("can only write up to 16 bytes");
    }

    let mut buffer = [0 as u8; 16];
    let n = args.len() - 3;
    for i in 0..n {
        let arg = try!(argv_parsed(args, i + 3, "BYTES", u8::parseint));
        buffer[i] = arg;
    }

    let location_arr = [loc];

    match TWI0.write(addr, &location_arr, &buffer[0..n as usize]) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.description())
    }
}

fn cmd_gpio_read(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        return Err("expected argument(s)");
    }
    let gpio_name = args[1];

    match pins::PIN_TABLE.iter().find(|&pin| {*(pin.name()) == *gpio_name}) {
        Some(pin) => println!("{}", pin.get()),
        None => println!("pin {} not found", gpio_name),
    }

    Ok(())
}

fn cmd_gpio_write(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 3 {
        return Err("expected argument(s)");
    }
    let gpio_name = args[1];
    let gpio_val = try!(argv_parsed(args, 2, "VALUE", i8::parseint));

    match pins::PIN_TABLE.iter().find(|&pin| {*(pin.name()) == *gpio_name}) {
        Some(pin) => pin.set(gpio_val != 0),
        None => println!("pin {} not found", gpio_name),
    }

    Ok(())
}

fn cmd_pwr_stat(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        return Err("expected argument(s)");
    }
    let supply_name = args[1];
    let _lock = supplies::POWER_MUTEX.lock();
    match supplies::SUPPLY_TABLE.iter().find(|&supply| {*(supply.name()) == *supply_name}) {
        Some(supply) => println!("supply {} status: {:?}", supply_name, try!(supply.status())),
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}

fn cmd_mount(_args: &[&str]) -> Result<(), &'static str>
{
    if !CARD.get() {
        return Err("card not found");
    }

    CARDEN.set(true);
    freertos::delay(1);
    let mut sd = SD.lock();

    match sd.check() {
        SdError::Ok => { return Ok(()); },
        e           => { println!("Error: {:?}", e); return Err("SD error"); }
    };
}

fn cmd_umount(_args: &[&str]) -> Result<(), &'static str>
{
    if !CARD.get() {
        return Err("card not found");
    }
    CARDEN.set(false);
    Ok(())
}

fn cmd_sdinfo(_args: &[&str]) -> Result<(), &'static str>
{
    if !CARD.get() {
        return Err("card not found");
    }

    let mut sd = SD.lock();

    println!("Type:      {:?}", sd.cardtype());
    println!("Version:   {:?}", sd.version());
    println!("Capacity:  {:?} MiB", sd.capacity() / 1024);
    println!("Protected: {}",
             if sd.writeprotected() { "yes" } else { "no" });

    Ok(())
}
