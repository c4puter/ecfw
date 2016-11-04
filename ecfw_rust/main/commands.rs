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
use main::{pins, supplies, reset};

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
    Command{ name: "srst",      f: cmd_srst,    descr: "soft reset" },
    Command{ name: "hrst",      f: cmd_hrst,    descr: "hard reset" },

    Command{ name: "i2c_probe", f: cmd_i2c_probe,   descr: "probe I2C for an ADDR" },
    Command{ name: "i2c_read",  f: cmd_i2c_read,    descr: "read I2C from ADDR at LOCATION, N bytes" },
    Command{ name: "i2c_write", f: cmd_i2c_write,   descr: "write I2C to ADDR at LOCATION, BYTES" },

    Command{ name: "gpio_read", f: cmd_gpio_read,   descr: "read GPIO (by name)" },
    Command{ name: "gpio_write",f: cmd_gpio_write,  descr: "write to GPIO (by name) VALUE" },

    Command{ name: "pwr_stat",  f: cmd_pwr_stat,    descr: "display status of SUPPLY" },
    Command{ name: "pwr_up",    f: cmd_pwr_up,      descr: "raise reference count of SUPPLY" },
    Command{ name: "pwr_dn",    f: cmd_pwr_dn,      descr: "lower reference count of SUPPLY" },
];

fn argv_parsed<T, U>(args: &[&str], n: usize, _name: &str, parser: fn(&str)->Result<T,U>) -> Result<T, &'static str>
    where U: fmt::Display
{
    let arg_parsed = match parser(args[n]) {
        Ok(val) => val,
        Err(_) => { return Err("cannot parse argument"); }
    };

    return Ok(arg_parsed);
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

fn cmd_srst(_args: &[&str]) -> Result<(), &'static str>
{
    reset::soft_reset();
    Err("did not reset")    // should never happen
}

fn cmd_hrst(_args: &[&str]) -> Result<(), &'static str>
{
    reset::hard_reset();
    Err("did not reset")    // should never happen
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

fn cmd_pwr_up(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        return Err("expected argument(s)");
    }
    let supply_name = args[1];
    let _lock = supplies::POWER_MUTEX.lock();
    match supplies::SUPPLY_TABLE.iter().find(|&supply| {*(supply.name()) == *supply_name}) {
        Some(supply) => println!("supply {} state changed? {}",
                                 supply_name, try!(supply.refcount_up())),
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}

fn cmd_pwr_dn(args: &[&str]) -> Result<(), &'static str>
{
    if args.len() < 2 {
        return Err("no supply specified");
    }
    let supply_name = args[1];
    let _lock = supplies::POWER_MUTEX.lock();
    match supplies::SUPPLY_TABLE.iter().find(|&supply| {*(supply.name()) == *supply_name}) {
        Some(supply) => println!("supply {} state changed? {}",
                                 supply_name, try!(supply.refcount_down())),
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}
