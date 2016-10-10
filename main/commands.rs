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

#![no_std]
#[macro_use]
extern crate ec_io;
extern crate freertos;
extern crate twi;
extern crate pins;
extern crate parseint;

use parseint::ParseInt;
use core::fmt;

pub trait Args<'a> {
    /// Return number of arguments, including argv[0]
    fn argc(&self) -> usize;

    /// Return the argument as &str, or None if it didn't validate as UTF-8
    fn argv(&self, n: usize) -> Option<&str>;
}

pub struct Command {
    pub name: &'static str,
    pub f: fn(args: &Args),
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
];

fn argv_parsed<T, U>(args: &Args, n: usize, name: &str, parser: fn(&str)->Result<T,U>) -> Option<T>
    where U: fmt::Display
{
    if n >= args.argc() {
        println!("expected argument {}", name);
        return None;
    }

    let arg_s = match args.argv(n) {
        Some(arg) => arg,
        None => { println!("cannot parse argument {}", name); return None; },
    };

    let arg_parsed = match parser(arg_s) {
        Ok(val) => val,
        Err(e) => { println!("argument parse error for {}: {}", name, e); return None; },
    };

    return Some(arg_parsed);
}

fn cmd_help(_args: &Args)
{
    for i in 0..COMMAND_TABLE.len() {
        let ref cmd = COMMAND_TABLE[i];
        println!("{:12} - {}", cmd.name, cmd.descr);
    }
}

fn cmd_free(_args: &Args)
{
    println!("{} B", freertos::get_free_heap());
}

fn cmd_i2c_probe(args: &Args)
{
    let addr = match argv_parsed(args, 1, "ADDR", u8::parseint) {
        Some(v) => v,
        None => return
    };
    match twi::twi0().probe(addr) {
        Ok(is_present) => {
            if is_present { println!("address {} present", addr); }
            else          { println!("address {} does not respond", addr); }
        },
        Err(s) => { println!("I2C error: {}", s); },
    };
}

fn cmd_i2c_read(args: &Args)
{
    let addr = match argv_parsed(args, 1, "ADDR", u8::parseint) {
        Some(v) => v,
        None => return };
    let loc = match argv_parsed(args, 2, "LOCATION", u8::parseint) {
        Some(v) => v,
        None => return };
    let n = match argv_parsed(args, 3, "N", u8::parseint) {
        Some(v) => v,
        None => return };
    if n > 16 {
        println!("can only read up to 16 bytes");
        return;
    }

    let location_arr = [loc];
    let mut buffer = [0 as u8; 16];

    match twi::twi0().read(addr, &location_arr, &mut buffer[0..n as usize]) {
        Ok(_) => {
            println!("{:?}", &buffer[0..n as usize]);
        }
        Err(s) => { println!("I2C error: {}", s); }
    }
}

fn cmd_i2c_write(args: &Args)
{
    let addr = match argv_parsed(args, 1, "ADDR", u8::parseint) {
        Some(v) => v,
        None => return
    };
    let loc = match argv_parsed(args, 2, "LOCATION", u8::parseint) {
        Some(v) => v,
        None => return
    };

    if args.argc() > 19 {
        println!("can only write up to 16 bytes");
        return;
    }

    let mut buffer = [0 as u8; 16];
    let n = args.argc() - 3;
    for i in 0..n {
        let arg = match argv_parsed(args, i + 3, "BYTES", u8::parseint) {
            Some(v) => v,
            None => return };
        buffer[i] = arg;
    }

    let location_arr = [loc];

    match twi::twi0().write(addr, &location_arr, &buffer[0..n as usize]) {
        Ok(_) => {}
        Err(s) => { println!("I2c error: {}", s); }
    }
}

fn cmd_gpio_read(args: &Args)
{
    let gpio_name = match args.argv(1) {
        Some(arg) => arg,
        None => { println!("GPIO not specified"); return; } };

    for &pin in pins::PIN_TABLE {
        if *(pin.name()) == *gpio_name {
            println!("{}", pin.get());
            return;
        }
    }

    println!("pin {} not found", gpio_name);
}

fn cmd_gpio_write(args: &Args)
{
    let gpio_name = match args.argv(1) {
        Some(arg) => arg,
        None => { println!("GPIO not specified"); return; } };
    let gpio_val = match argv_parsed(args, 2, "VALUE", i8::parseint) {
        Some(v) => v,
        None => return };

    for &pin in pins::PIN_TABLE {
        if *(pin.name()) == *gpio_name {
            pin.set(gpio_val != 0);
            return;
        }
    }

    println!("pin {} not found", gpio_name);
}
