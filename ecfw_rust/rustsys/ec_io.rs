/*
 * c4puter embedded controller firmware
 * Copyright (C) 2017 Chris Pavlina
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

use core::fmt;
extern crate bindgen_mcu;
extern crate asf_usart;
use os;

#[allow(dead_code)]
const USART0: *mut asf_usart::Usart = 0x40024000u32 as *mut asf_usart::Usart;
const USART1: *mut asf_usart::Usart = 0x40028000u32 as *mut asf_usart::Usart;

const USART_DBG: *mut asf_usart::Usart = USART1;

struct UartWriter {}
struct UartWriterAsync {}

queue_static_new! {
    STDIN_QUEUE: [u8; 1024];
    STDOUT_QUEUE: [u8; 72];
}

static STDOUT_MUTEX: os::Mutex<()> = os::Mutex::new(());

fn putc_task(q: &'static os::Queue<'static, u8>)
{
    q.register_receiver();
    loop {
        let c = q.receive_wait_blocking();
        unsafe {
            //asf_usart::usart_putchar(USART_DBG, c as u32);
            bindgen_mcu::mcu_usb_putchar(c as i8);
        }
    }
}

pub fn putc(c: u8) {
    STDOUT_QUEUE.send_wait(c);
}

pub fn putc_async(c: u8) {
    //unsafe{asf_usart::usart_putchar(USART_DBG, c as u32)};
    unsafe{bindgen_mcu::mcu_usb_putchar(c as i8)};
}

pub fn getc_async() -> u8 {
    loop {
        let c = unsafe{bindgen_mcu::mcu_usb_getchar()};
        if c > 0 && c <= 255 {
            return c as u8;
        }
    }
    //STDIN_QUEUE.receive_wait()
}

pub fn flush_output() {
    STDOUT_QUEUE.flush();
}


#[no_mangle]
#[allow(unreachable_code,non_snake_case)]
pub extern "C" fn USART1_Handler() {
    let status = unsafe{asf_usart::usart_get_status(USART_DBG)};
    if status & (asf_usart::US_CSR_RXRDY as u32) != 0 {
        let mut rxdata = 0u32;
        unsafe{asf_usart::usart_read(USART_DBG, &mut rxdata)};
        if rxdata > 0 && rxdata <= 255 {
            let rxbyte = rxdata as u8;
            if !STDIN_QUEUE.send_no_wait(rxbyte) {
                println_async(format_args!("\n\nUSART BUFFER OVERFLOW\n"));
            }
        }
    }
}

#[no_mangle]
#[allow(unreachable_code,non_snake_case)]
pub extern "C" fn callback_cdc_rx_notify(_: u8) { }
/*
    let c = unsafe{bindgen_mcu::mcu_usb_getchar()};
    if c > 0 && c <= 255 {
        let rxbyte = c as u8;
        if !STDIN_QUEUE.send_no_wait(rxbyte) {
            println_async(format_args!("\n\nUSART BUFFER OVERFLOW\n"));
        }
    }
}
*/

impl<'a> fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            if *c == b'\n' {
                putc(b'\r');
            }
            putc(*c);
        }
        return Ok(());
    }
}

impl<'a> fmt::Write for UartWriterAsync {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            if *c == b'\n' {
                putc_async(b'\r');
            }
            putc_async(*c);
        }
        return Ok(());
    }
}

/// Initialize EC IO. Starts a FreeRTOS task.
pub fn init()
{
    let usart_settings = asf_usart::sam_usart_opt_t {
        baudrate: 115200,
        char_length: asf_usart::US_MR_CHRL_8_BIT as u32,
        parity_type: asf_usart::US_MR_PAR_NO as u32,
        stop_bits: asf_usart::US_MR_NBSTOP_1_BIT as u32,
        channel_mode: asf_usart::US_MR_CHMODE_NORMAL as u32,
        irda_filter: 0,
    };

    unsafe {
        asf_usart::usart_init_rs232(
            USART_DBG, &usart_settings, bindgen_mcu::mcu_get_peripheral_hz());
        asf_usart::usart_enable_tx(USART_DBG);
        asf_usart::usart_enable_rx(USART_DBG);
        asf_usart::usart_enable_interrupt(USART_DBG, asf_usart::US_IER_RXRDY as u32);
        bindgen_mcu::mcu_set_irq_prio(asf_usart::IRQn_USART1_IRQn as i32, 4, 1);
        bindgen_mcu::mcu_enable_irq(asf_usart::IRQn_USART1_IRQn as i32);
    }
    os::Task::new(move || { putc_task(&STDOUT_QUEUE); }, "ec_io", 200, 0);
}

fn _print(args: fmt::Arguments) {
    fmt::write(&mut UartWriter{}, args).unwrap();
}

pub fn print(args: fmt::Arguments) {
    let _lock = STDOUT_MUTEX.lock();
    _print(args);
}

pub fn println(args: fmt::Arguments) {
    let _lock = STDOUT_MUTEX.lock();
    _print(args);
    fmt::Write::write_str(&mut UartWriter{}, "\n").unwrap();
}

pub fn print_async(args: fmt::Arguments) {
    fmt::write(&mut UartWriterAsync{}, args).unwrap();
}

pub fn println_async(args: fmt::Arguments) {
    print_async(args);
    fmt::Write::write_str(&mut UartWriterAsync{}, "\n").unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::rustsys::ec_io::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ($crate::rustsys::ec_io::println(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_async {
    ($($arg:tt)*) => ($crate::rustsys::ec_io::print_async(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_async {
    ($($arg:tt)*) => ($crate::rustsys::ec_io::println_async(format_args!($($arg)*)));
}
