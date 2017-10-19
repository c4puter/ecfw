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

use rustsys::rust_support::*;
use core::fmt;

#[lang = "panic_fmt"]
pub extern "C" fn panic_impl(
    fmt: fmt::Arguments,
    file: &'static str,
    line: u32,
) -> !
{
    unsafe {
        disable_irq();
    }
    print_async!("\n\n===================================\n");
    print_async!("PANIC\n");
    print_async!("file:line = {}:{}\n", file, line);
    print_async!("message   = {}\n", fmt);
    loop {}
}

#[no_mangle]
pub extern "C" fn hard_fault_printer(regs: [u32; 8])
{
    unsafe {
        disable_irq();
    }
    print_async!("\n\n===================================\n");
    print_async!("HARD FAULT\n");
    print_async!(
        "r0  = {:08x}  r1  = {:08x}  r2  = {:08x}  r3  = {:08x}\n",
        regs[0],
        regs[1],
        regs[2],
        regs[3]
    );
    print_async!(
        "r12 = {:08x}  lr  = {:08x}  pc  = {:08x}  psr = {:08x}\n",
        regs[4],
        regs[5],
        regs[6],
        regs[7]
    );

    print_async!("\n");
    unsafe {
        print_async!(
            "BFAR = {:08x}  CFSR = {:08x}  HFSR = {:08x}\n",
            readmem(0xe000ed38),
            readmem(0xe000ed28),
            readmem(0xe000ed2c)
        );
        print_async!(
            "DFSR = {:08x}  AFSR = {:08x}  SCB_SHCSR = {:08x}\n",
            readmem(0xe000ed30),
            readmem(0xe000ed3c),
            readmem(0xe000e000 + 0x0d00 + 0x024)
        );
        //     SCS_BASE    SCB_off   SHCSR_off
    }
    loop {}
}

#[no_mangle]
pub extern "C" fn freertos_assert()
{
    unsafe {
        disable_irq();
    }
    print_async!("\n\n===================================\n");
    print_async!("PANIC\n");
    print_async!("FreeRTOS assertion failure\n");
    loop {}
}
