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

extern crate bindgen_mcu;
extern crate bindgen_usart;

extern crate rust_support;
extern crate panicking;
#[macro_use]
extern crate ec_io;
extern crate twi;
extern crate pins;
extern crate ledmatrix;
extern crate freertos;
extern crate esh;
extern crate commands;

use core::str;

struct EshArgAdapter<'a> {
    argarray: &'a esh::EshArgArray
}

impl<'a> commands::Args<'a> for EshArgAdapter<'a> {
    fn argc(&self) -> usize {
        return self.argarray.len();
    }
    fn argv(&self, n: usize) -> Option<&str> {
        if n >= self.argarray.len() {
            return None;
        }
        return match str::from_utf8(&self.argarray[n]) {
            Ok(s) => Some(s),
            _ => None,
        };
    }
}

fn command_dispatch(_esh: &esh::Esh, args: &esh::EshArgArray)
{
    if args.len() >= 1 {
        let argadapter = EshArgAdapter{argarray: args};
        let argv0 = match str::from_utf8(&args[0]) {
            Ok(s) => s,
            _ => "__invalid_command",
        };

        let mut f: Option<fn(&commands::Args)> = None;

        for i in 0..(commands::COMMAND_TABLE.len()) {
            let ref cmd = commands::COMMAND_TABLE[i];
            if *(cmd.name) == *argv0 {
                f = Some(cmd.f);
                break;
            }
        }

        match f {
            Some(f) => f(&argadapter),
            None => println!("unrecognized command: {}", argv0),
        };
    }
}

fn esh_print_cb(_esh: &esh::Esh, c: u8)
{
    if c == b'\n' {
        ec_io::putc(b'\r');
    }
    ec_io::putc(c);
}

pub fn esh_task() {
    let mut esh = esh::Esh::init().unwrap();
    esh.register_command(command_dispatch);
    esh.register_print(esh_print_cb);

    loop {
        let c = ec_io::getc_async();
        let c_replaced =
            if c == b'\r' { b'\n' }
            else          { c };
        esh.rx(c_replaced);
    }
}

pub fn init_task()
{
    println!("Hand off to init task");
    println!("Initialize LED matrix...");
    unsafe{ ledmatrix::matrix_init(&pins::U801).unwrap(); }
    freertos::delay(500);
    ledmatrix::matrix().set_all(false).unwrap();

    println_async!("Create task \"esh\"...");
    freertos::Task::new(move || { esh_task() }, "esh", 1000, 0);

    loop {
        /*
        ledmatrix::matrix().set_led(0xb1, true).unwrap();
        freertos::delay(250);
        ledmatrix::matrix().set_led(0xb1, false).unwrap();
        freertos::delay(250);
        */
    }
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern "C" fn main() -> i32 {
    unsafe {
        bindgen_mcu::mcu_init();
        bindgen_mcu::board_init();
    }

    ec_io::init();
    println_async!("Initialized EC core and USART");

    println_async!("Initialize I2C...");
    unsafe{ twi::twi0_init(400000).unwrap(); }

    println_async!("Initialize GPIO...");
    for &pin in pins::PIN_TABLE {
        pin.init();
    }

    println_async!("Create task \"init\"...");
    freertos::Task::new(move || { init_task() }, "init", 500, 0);

    println_async!("Start task scheduler...");
    freertos::run();

    loop {}

    return 0;
}
