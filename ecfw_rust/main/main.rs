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

use main::{commands, sysman, reset};
use esh;
use drivers;
use drivers::gpio::Gpio;
use devices;
use bindgen_mcu;
use rustsys::ec_io;
use os;

use core::str;

fn command_dispatch(_esh: &esh::Esh, args: &[&str])
{
    if args.len() < 1 {
        return;
    }

    match commands::COMMAND_TABLE.iter().find(|&c| {*(c.name) == *args[0]}) {
        Some(cmd) =>
            if let Err(s) = (cmd.f)(args) {
                println!("error: {}", s.message);
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
    debug!(DEBUG_ECBOOT, "start debug console");
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
    let unused = unsafe{bindgen_mcu::get_stack_unused()};
    debug!(DEBUG_ECBOOT, "main stack unused: {} bytes", unused);

    ec_io::flush_output();
    debug!(DEBUG_ECBOOT, "initialize TWI");
    devices::twi::TWI0.init(400000).unwrap();

    ec_io::flush_output();
    debug!(DEBUG_ECBOOT, "initialize GPIO");
    for &pin in devices::pins::PIN_TABLE {
        pin.init();
    }

    // Power supply safety can be released once pins are initialized
    devices::pins::EN_SAFETY.set(false);

    // Put all power supplies in known state - all but standby rail down
    reset::shutdown_supplies_cleanly();

    ec_io::flush_output();
    debug!(DEBUG_ECBOOT, "initialize LED matrix");
    devices::MATRIX.write().init().unwrap();
    os::delay(250);
    {
        let mut mat = devices::MATRIX.write();
        mat.buffer_all(false);
        mat.flush().unwrap();
    }

    ec_io::flush_output();
    debug!(DEBUG_ECBOOT, "initialize HSMCI (SD)");
    drivers::sd::init();

    os::Task::new(sysman::run_event, "event", 500, 0);
    os::Task::new(sysman::run_status, "status", 500, 0);
    os::yield_task(); // Let above tasks emit status messages

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
    debug_async!(DEBUG_ECBOOT, "==================================================");
    debug_async!(DEBUG_ECBOOT, "# Booting EC firmware");
    match option_env!("BUILD_ID") {
        Some(s) => debug_async!(DEBUG_ECBOOT, "# Build ID: {}", s),
        None    => debug_async!(DEBUG_ECBOOT, "# No build ID"),
    };
    debug_async!(DEBUG_ECBOOT, "==================================================");
    debug_async!(DEBUG_ECBOOT, "");
    debug_async!(DEBUG_ECBOOT, "initialized EC core and USART");

    os::Task::new(init_task, "init", 10000, 0);

    ec_io::flush_output();
    debug_async!(DEBUG_ECBOOT, "start scheduler and hand off to init task...");
    os::freertos::run();

    loop {}

    return 0;
}
