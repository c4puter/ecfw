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

pub trait Args<'a> {
    // Return number of arguments, including argv[0]
    fn argc(&self) -> usize;

    // Return the argument as &str, or None if it didn't validate as UTF-8
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
];

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
