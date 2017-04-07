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

use main::{commands, pins, sysman};
use esh;
use hardware::{ledmatrix, twi};
use hardware::gpio::Gpio;
use bindgen_mcu;
use rustsys::{ec_io,freertos};

use core::str;

fn command_dispatch(_esh: &esh::Esh, args: &[&str])
{
    if args.len() < 1 {
        return;
    }

    match commands::COMMAND_TABLE.iter().find(|&c| {*(c.name) == *args[0]}) {
        Some(cmd) =>
            if let Err(s) = (cmd.f)(args) {
                println!("error: {}", s);
            },
        None => println!("unrecognized command: {}", args[0]),
    }
}

fn esh_print_cb(_esh: &esh::Esh, c: char)
{
    if c == '\n' {
        print!("\r");
    }
    print!("{}", c);
}

pub fn esh_task() {
    println!("Start debug console");
    let mut esh = esh::Esh::init().unwrap();
    esh.register_command(command_dispatch);
    esh.register_print(esh_print_cb);
    esh.rx(b'\n');

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
    println!("success");

    println!("Main stack unused: {} bytes",
             unsafe{bindgen_mcu::get_stack_unused()});

    println!("Initialize I2C");
    twi::TWI0.init(400000).unwrap();

    println!("Initialize GPIO");
    for &pin in pins::PIN_TABLE {
        pin.init();
    }

    // Power supply safety can be released once pins are initialized
    pins::EN_SAFETY.set(false);
    println!("Initialize LED matrix");
    unsafe{ ledmatrix::matrix_init(&pins::U801).unwrap(); }
    freertos::delay(250);
    ledmatrix::matrix().set_all(false).unwrap();

    freertos::Task::new(sysman::run_event, "event", 500, 0);
    freertos::Task::new(sysman::run_status, "status", 500, 0);
    freertos::yield_task(); // Let above tasks emit status messages

    // Don't run esh_task() as a task; we can't free heap, so if we just spin
    // forever we've wasted the init_task heap. "exec" it instead.

    esh_task();
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern "C" fn main() -> i32 {
    unsafe {
        bindgen_mcu::mcu_init();
        bindgen_mcu::write_stack_canaries();
    }

    ec_io::init();
    println_async!("");
    println_async!("==================================================");
    println_async!("# Booting EC firmware");
    match option_env!("BUILD_ID") {
        Some(s) => println_async!("# Build ID: {}", s),
        None    => println_async!("# No build ID"),
    };
    println_async!("==================================================");
    println_async!("");
    println_async!("Initialized EC core and USART");

    freertos::Task::new(init_task, "init", 1000, 0);

    print_async!("Start scheduler and hand off to init task...");
    freertos::run();

    loop {}

    return 0;
}
