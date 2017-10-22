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

//! USB-CDC COM driver (wrapper around Atmel ASF's USB-CDC).

use asf_udc;
use asf_udi_cdc;
use ctypes::c_void;

use os;
use core::sync::atomic::*;
use drivers::com::Com;

static CDC_ENABLED: AtomicBool = ATOMIC_BOOL_INIT;
static CDC_CONFIGURED: AtomicBool = ATOMIC_BOOL_INIT;

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
        ComCdc::putchar(c);
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
        unsafe { asf_udc::udc_start() };
    }

    pub fn stop(&self)
    {
        unsafe { asf_udc::udc_stop() };
    }

    fn putchar(c: u8)
    {
        if CDC_ENABLED.load(Ordering::Relaxed) {
            unsafe {
                asf_udi_cdc::udi_cdc_putc(c as i32);
            }
        }
    }

    fn getchar() -> Option<u8>
    {
        if CDC_CONFIGURED.load(Ordering::Relaxed) {
            if unsafe {asf_udi_cdc::udi_cdc_is_rx_ready() } {
                unsafe {
                    Some(asf_udi_cdc::udi_cdc_getc() as u8)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Com for ComCdc {
    fn getc(&self) -> Option<u8>
    {
        ComCdc::getchar()
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
        ComCdc::putchar(c);
    }

    fn flush_output(&self) -> bool
    {
        self.queue_out.flush();
        true
    }
}

#[no_mangle]
pub extern "C" fn callback_cdc_enable(_port: u8) -> bool
{
    CDC_ENABLED.store(true, Ordering::Relaxed);
    true
}

#[no_mangle]
pub extern "C" fn callback_cdc_disable(_port: u8)
{
    CDC_ENABLED.store(false, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn callback_cdc_set_coding_ext(_port: u8, _cfg: *const c_void)
{
    CDC_CONFIGURED.store(true, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn callback_cdc_set_dtr(_port: u8, _enable: bool)
{
    CDC_CONFIGURED.store(true, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn main_sof_action() {}

#[no_mangle]
pub extern "C" fn main_resume_action() {}

#[no_mangle]
pub extern "C" fn main_suspend_action() {}
