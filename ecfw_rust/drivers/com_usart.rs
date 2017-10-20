// c4puter embedded controller firmware
// Copyright (C) 2017 Chris Pavlina
//
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

//! COM driver using on-chip USART.

use bindgen_mcu;
use asf_usart;
use os;
use drivers::com::Com;

#[allow(dead_code)]
const USART0: *mut asf_usart::Usart = 0x40024000u32 as *mut asf_usart::Usart;
const USART1: *mut asf_usart::Usart = 0x40028000u32 as *mut asf_usart::Usart;

pub struct ComUsart {
    usart: *mut asf_usart::Usart,
    irqn: i32,
    queue_in: &'static os::Queue<'static, u8>,
    queue_out: &'static os::Queue<'static, u8>,
}

queue_static_new! {
    QUEUE_IN_1: [u8; 1024];
    QUEUE_OUT_1: [u8; 1024];
}

/// COM over local USART (for debug, to RS232 on board)
pub static COMUSART: ComUsart = ComUsart {
    usart: USART1,
    irqn: asf_usart::IRQn::USART1_IRQn as i32,
    queue_in: &QUEUE_IN_1,
    queue_out: &QUEUE_OUT_1,
};

fn putc_task(q: &'static os::Queue<'static, u8>, usart: *mut asf_usart::Usart)
{
    q.register_receiver();
    loop {
        let c = q.receive_wait_blocking();
        unsafe {
            asf_usart::usart_putchar(usart, c as u32);
        }
    }
}

impl ComUsart {
    pub unsafe fn init(&self, baud: u32)
    {
        let usart_settings = asf_usart::sam_usart_opt_t {
            baudrate: baud,
            char_length: asf_usart::US_MR_CHRL_8_BIT as u32,
            parity_type: asf_usart::US_MR_PAR_NO as u32,
            stop_bits: asf_usart::US_MR_NBSTOP_1_BIT as u32,
            channel_mode: asf_usart::US_MR_CHMODE_NORMAL as u32,
            irda_filter: 0,
        };

        os::Task::new(
            move || { putc_task(&self.queue_out, self.usart); },
            "comusart",
            200,
            0,
        );

        let fcpu = bindgen_mcu::mcu_get_peripheral_hz();
        asf_usart::usart_init_rs232(self.usart, &usart_settings, fcpu);
        asf_usart::usart_enable_tx(self.usart);
        asf_usart::usart_enable_rx(self.usart);
        asf_usart::usart_enable_interrupt(
            self.usart,
            asf_usart::US_IER_RXRDY as u32,
        );
        bindgen_mcu::mcu_set_irq_prio(self.irqn, 4, 1);
        bindgen_mcu::mcu_enable_irq(self.irqn);
    }
}

impl Com for ComUsart {
    fn getc(&self) -> Option<u8>
    {
        self.queue_in.receive_no_wait()
    }

    fn putc(&self, c: u8)
    {
        self.queue_out.send_wait(c);
    }

    fn putc_nowait(&self, c: u8)
    {
        self.queue_out.send_no_wait(c);
    }

    fn putc_async(&self, c: u8)
    {
        unsafe { asf_usart::usart_putchar(self.usart, c as u32) };
    }

    fn flush_output(&self) -> bool
    {
        self.queue_out.flush();
        true
    }
}

unsafe impl Sync for ComUsart {}

macro_rules! define_usart_isr {
    ( $handler:ident, $comusart:expr ) => (
        #[no_mangle]
        #[allow(unreachable_code,non_snake_case)]
        pub extern "C" fn $handler() {
            let status = unsafe{asf_usart::usart_get_status($comusart.usart)};
            if status & (asf_usart::US_CSR_RXRDY as u32) != 0 {
                let mut rxdata = 0;
                unsafe {
                    asf_usart::usart_read($comusart.usart, &mut rxdata);
                }
                if rxdata > 0 && rxdata <= 255 {
                    if !$comusart.queue_in.send_no_wait(rxdata as u8) {
                        print_to_async!(&$comusart, "\n\nUSART BUFFER OVERFLOW\n\n");
                    }
                }
            }
        }
    );
}

define_usart_isr!(USART1_Handler, COMUSART);
