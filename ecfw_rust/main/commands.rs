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
use hardware::twi;
use main::{pins, supplies};

use main::parseint::ParseInt;
use esh::{EshArgArray,Utf8Error};
use core::fmt;
use core::convert::From;

pub struct Command {
    pub name: &'static str,
    pub f: fn(args: &EshArgArray) -> Result<(), &'static str>,
    pub descr: &'static str,
}

pub static COMMAND_TABLE: &'static [Command] = &[
    Command{ name: "help",      f: cmd_help,    descr: "display commands and their descriptions" },
    Command{ name: "free",      f: cmd_free,    descr: "display free heap" },

    Command{ name: "i2c_probe", f: cmd_i2c_probe,   descr: "probe I2C for an ADDR" },
    Command{ name: "i2c_read",  f: cmd_i2c_read,    descr: "read I2C from ADDR at LOCATION, N bytes" },
    Command{ name: "i2c_write", f: cmd_i2c_write,   descr: "write I2C to ADDR at LOCATION, BYTES" },

    Command{ name: "gpio_read", f: cmd_gpio_read,   descr: "read GPIO (by name)" },
    Command{ name: "gpio_write",f: cmd_gpio_write,  descr: "write to GPIO (by name) VALUE" },

    Command{ name: "pwr_stat",  f: cmd_pwr_stat,    descr: "display status of SUPPLY" },
    Command{ name: "pwr_up",    f: cmd_pwr_up,      descr: "raise reference count of SUPPLY" },
    Command{ name: "pwr_dn",    f: cmd_pwr_dn,      descr: "lower reference count of SUPPLY" },
];

macro_rules! try_utf8 {
    ( $e:expr ) => (
        match $e {
            Ok(v) => v,
            Err(_) => return Err("cannot parse UTF-8") } )
}

fn argv_parsed<T, U>(args: &EshArgArray, n: usize, _name: &str, parser: fn(&str)->Result<T,U>) -> Result<T, &'static str>
    where U: fmt::Display
{
    let arg_s = try_utf8!(args.get_str(n));

    let arg_parsed = match parser(arg_s) {
        Ok(val) => val,
        Err(_) => { return Err("cannot parse argument"); }
    };

    return Ok(arg_parsed);
}

fn cmd_help(_args: &EshArgArray) -> Result<(), &'static str>
{
    for i in 0..COMMAND_TABLE.len() {
        let ref cmd = COMMAND_TABLE[i];
        println!("{:12} - {}", cmd.name, cmd.descr);
    }

    Ok(())
}

fn cmd_free(_args: &EshArgArray) -> Result<(), &'static str>
{
    println!("{} B", freertos::get_free_heap());

    Ok(())
}

fn cmd_i2c_probe(args: &EshArgArray) -> Result<(), &'static str>
{
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));
    match twi::twi0().probe(addr) {
        Ok(is_present) => {
            if is_present { println!("address {} present", addr); }
            else          { println!("address {} does not respond", addr); }
            Ok(())
        },
        Err(e) => { Err(e.description()) }
    }
}

fn cmd_i2c_read(args: &EshArgArray) -> Result<(), &'static str>
{
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));
    let loc = try!(argv_parsed(args, 2, "LOCATION", u8::parseint));
    let n = try!(argv_parsed(args, 3, "N", u8::parseint));
    if n > 16 {
        return Err("can only read up to 16 bytes");
    }

    let location_arr = [loc];
    let mut buffer = [0 as u8; 16];

    match twi::twi0().read(addr, &location_arr, &mut buffer[0..n as usize]) {
        Ok(_) => {
            println!("{:?}", &buffer[0..n as usize]);
            Ok(())
        }
        Err(e) => Err(e.description())
    }
}

fn cmd_i2c_write(args: &EshArgArray) -> Result<(), &'static str>
{
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

    match twi::twi0().write(addr, &location_arr, &buffer[0..n as usize]) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.description())
    }
}

fn cmd_gpio_read(args: &EshArgArray) -> Result<(), &'static str>
{
    let gpio_name = try_utf8!(args.get_str(1));

    match pins::PIN_TABLE.iter().find(|&pin| {*(pin.name()) == *gpio_name}) {
        Some(pin) => println!("{}", pin.get()),
        None => println!("pin {} not found", gpio_name),
    }

    Ok(())
}

fn cmd_gpio_write(args: &EshArgArray) -> Result<(), &'static str>
{
    let gpio_name = try_utf8!(args.get_str(1));
    let gpio_val = try!(argv_parsed(args, 2, "VALUE", i8::parseint));

    match pins::PIN_TABLE.iter().find(|&pin| {*(pin.name()) == *gpio_name}) {
        Some(pin) => pin.set(gpio_val != 0),
        None => println!("pin {} not found", gpio_name),
    }

    Ok(())
}

fn cmd_pwr_stat(args: &EshArgArray) -> Result<(), &'static str>
{
    let supply_name = try_utf8!(args.get_str(1));
    match supplies::SUPPLY_TABLE.iter().find(|&supply| {*(supply.name()) == *supply_name}) {
        Some(supply) => println!("supply {} up? {}", supply_name, try!(supply.is_up())),
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}

fn cmd_pwr_up(args: &EshArgArray) -> Result<(), &'static str>
{
    let supply_name = try_utf8!(args.get_str(1));
    match supplies::SUPPLY_TABLE.iter().find(|&supply| {*(supply.name()) == *supply_name}) {
        Some(supply) => println!("supply {} state changed? {}",
                                 supply_name, try!(supply.refcount_up())),
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}

fn cmd_pwr_dn(args: &EshArgArray) -> Result<(), &'static str>
{
    let supply_name = try_utf8!(args.get_str(1));
    match supplies::SUPPLY_TABLE.iter().find(|&supply| {*(supply.name()) == *supply_name}) {
        Some(supply) => println!("supply {} state changed? {}",
                                 supply_name, try!(supply.refcount_down())),
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}
