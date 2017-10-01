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
#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(asm)]
#![feature(alloc)]
#![feature(allocator)]
#![feature(heap_api)]

#![feature(plugin)]
#![plugin(repeat)]

#![crate_type = "rlib"]
#![allocator]

extern crate esh;
extern crate alloc;
extern crate bindgen_mcu;

#[macro_use] pub mod os;
#[macro_use] pub mod rustsys;
#[macro_use] pub mod messages;
#[macro_use] pub mod drivers;
#[macro_use] pub mod devices;
#[macro_use] pub mod data;
#[macro_use] pub mod main;
