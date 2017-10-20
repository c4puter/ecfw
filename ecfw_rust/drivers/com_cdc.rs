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

extern crate bindgen_mcu;
use os;
use drivers::com::Com;

pub struct ComCdc {
    queue_out: &'static os::Queue<'static, u8>,
}

queue_static_new! {
    QUEUE_OUT: [u8; 1024];
}

/// COM over USB-CDC (for debug, to USB port on board)
///
/// Will extend to support multiple logical ports later; one will remain
/// for debug and the other will be passed through to the main system.
pub static COMCDC: ComCdc = ComCdc { queue_out: &QUEUE_OUT };

fn putc_task(q: &'static os::Queue<'static, u8>)
{
    q.register_receiver();
    loop {
        let c = q.receive_wait_blocking();
        unsafe {
            bindgen_mcu::mcu_usb_putchar(c as i8);
        }
    }
}

impl ComCdc {
    pub fn init(&self)
    {
        os::Task::new(
            move || { putc_task(&self.queue_out); },
            "comcdc",
            200,
            0,
        );
    }

    pub fn start(&self)
    {
        unsafe { bindgen_mcu::mcu_start_usb() };
    }

    pub fn stop(&self)
    {
        unsafe { bindgen_mcu::mcu_stop_usb() };
    }
}

impl Com for ComCdc {
    fn getc(&self) -> Option<u8>
    {
        let c = unsafe { bindgen_mcu::mcu_usb_getchar() };
        if c > 0 && c <= 255 {
            Some(c as u8)
        } else {
            None
        }
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
        unsafe {
            bindgen_mcu::mcu_usb_putchar(c as i8);
        }
    }

    fn flush_output(&self) -> bool
    {
        self.queue_out.flush();
        true
    }
}
