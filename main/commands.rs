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

use core::str::FromStr;
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
    Command{ name: "args",      f: cmd_args,    descr: "test: print out all arguments" },
    Command{ name: "free",      f: cmd_free,    descr: "display free heap" },

    Command{ name: "i2c_init",  f: cmd_i2c_init,    descr: "initialize I2C at FREQUENCY" },
    Command{ name: "i2c_probe", f: cmd_i2c_probe,   descr: "probe I2C for an ADDR" },
    Command{ name: "i2c_read",  f: cmd_i2c_read,    descr: "read I2C from ADDR at LOCATION, N bytes" },
    Command{ name: "i2c_write", f: cmd_i2c_write,   descr: "write I2C to ADDR at LOCATION, BYTES" },
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

fn cmd_args(args: &Args)
{
    for i in 0..args.argc() {
        match args.argv(i) {
            Some(arg) => println!("argv[{:2}] = {}", i, arg),
            None => println!("argv[{:2}] is invalid UTF-8", i),
        };
    }
}

fn cmd_free(_args: &Args)
{
    println!("{} B", freertos::get_free_heap());
}

static mut I2C: Option<twi::Twi> = None;

fn cmd_i2c_init(args: &Args)
{
    let freq = match argv_parsed(args, 1, "FREQ", u32::from_str) {
        Some(v) => v,
        None => return
    };
    match unsafe{I2C.clone()} {
        Some(_) => { println!("I2C already initialized!"); },
        None => {
            let i2c = twi::Twi::new(twi::TWI0);
            match i2c.init(freq) {
                Ok(_) => unsafe{I2C = Some(i2c);},
                Err(s) => {println!("initialization error: {}", s);},
            }
        }
    };
}

fn cmd_i2c_probe(args: &Args)
{
    let addr = match argv_parsed(args, 1, "ADDR", u8::from_str) {
        Some(v) => v,
        None => return
    };
    match unsafe{I2C.clone()} {
        None => {
            println!("I2C must be initialized first!");
        }
        Some(i2c) => {
            match i2c.probe(addr) {
                Ok(is_present) => {
                    if is_present { println!("address {} present", addr); }
                    else          { println!("address {} does not respond", addr); }
                },
                Err(s) => { println!("I2C error: {}", s); },
            };
        }
    }
}

fn cmd_i2c_read(args: &Args)
{
    let addr = match argv_parsed(args, 1, "ADDR", u8::from_str) {
        Some(v) => v,
        None => return };
    let loc = match argv_parsed(args, 2, "LOCATION", u8::from_str) {
        Some(v) => v,
        None => return };
    let n = match argv_parsed(args, 3, "N", u8::from_str) {
        Some(v) => v,
        None => return };
    match unsafe{I2C.clone()} {
        None => {
            println!("I2C must be initialized first!");
        }
        Some(i2c) => {
            if n > 16 {
                println!("can only read up to 16 bytes");
                return;
            }

            let location_arr = [loc];
            let mut buffer = [0 as u8; 16];

            match i2c.read(addr, &location_arr, &mut buffer[0..n as usize]) {
                Ok(_) => {
                    println!("{:?}", &buffer[0..n as usize]);
                }
                Err(s) => { println!("I2C error: {}", s); }
            }
        }
    }
}

fn cmd_i2c_write(args: &Args)
{
    let addr = match argv_parsed(args, 1, "ADDR", u8::from_str) {
        Some(v) => v,
        None => return
    };
    let loc = match argv_parsed(args, 1, "LOCATION", u8::from_str) {
        Some(v) => v,
        None => return
    };

    match unsafe{I2C.clone()} {
        None => {
            println!("I2C must be initialized first!");
        }
        Some(i2c) => {
            if args.argc() > 19 {
                println!("can only write up to 16 bytes");
                return;
            }

            let mut buffer = [0 as u8; 16];
            let n = args.argc() - 3;
            for i in 0..n {
                let arg = match argv_parsed(args, i + 3, "BYTES", u8::from_str) {
                    Some(v) => v,
                    None => return };
                buffer[i] = arg;
            }

            let location_arr = [loc];

            match i2c.write(addr, &location_arr, &buffer[0..n as usize]) {
                Ok(_) => {}
                Err(s) => { println!("I2c error: {}", s); }
            }
        }
    }
}
