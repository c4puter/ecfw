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

#![no_std]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(asm)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(heap_api)]
#![feature(const_cell_new)]
#![feature(const_unsafe_cell_new)]
#![feature(const_atomic_bool_new)]

#![feature(plugin)]
#![plugin(repeat)]

#![crate_type = "rlib"]

extern crate alloc;
extern crate ctypes;
extern crate esh;
extern crate lwext4;
extern crate lwext4_crc32;

extern crate bindgen_mcu;

extern crate asf_pio;
extern crate asf_rstc;
extern crate asf_pdc;
extern crate asf_sd_mmc;
extern crate asf_usart;
extern crate asf_udc;
extern crate asf_udi_cdc;

#[macro_use] pub mod os;
#[macro_use] pub mod rustsys;
#[macro_use] pub mod messages;
#[macro_use] pub mod drivers;
#[macro_use] pub mod devices;
#[macro_use] pub mod data;
#[macro_use] pub mod main;
