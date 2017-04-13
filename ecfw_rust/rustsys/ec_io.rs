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
    STDIN_QUEUE: [u8; 256];
    STDOUT_QUEUE: [u8; 72];
}

static STDOUT_MUTEX: os::Mutex<()> = os::Mutex::new(());

fn putc_task(q: &'static os::Queue<'static, u8>)
{
    q.register_receiver();
    loop {
        let c = q.receive_wait_blocking();
        unsafe {
            asf_usart::usart_putchar(USART_DBG, c as u32);
        }
    }
}

pub fn putc(c: u8) {
    STDOUT_QUEUE.send_wait(c);
}

pub fn putc_async(c: u8) {
    unsafe{asf_usart::usart_putchar(USART_DBG, c as u32)};
}

pub fn getc_async() -> u8 {
    STDIN_QUEUE.receive_wait()
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
        bindgen_mcu::mcu_set_irq_prio(asf_usart::IRQn::USART1_IRQn as i32, 4, 1);
        bindgen_mcu::mcu_enable_irq(asf_usart::IRQn::USART1_IRQn as i32);
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
