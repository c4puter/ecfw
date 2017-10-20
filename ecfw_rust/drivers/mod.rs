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

//! Drivers for both hardware and software (e.g. filesystem)

pub mod ext4;
pub mod gpio;
pub mod ledmatrix;
pub mod sd;
pub mod tempsensor;
pub mod clocksynth;
pub mod twi;
pub mod spi;
pub mod fpga;
pub mod northbridge;
pub mod gpt;
pub mod power;
pub mod ftrans;
#[macro_use] pub mod com;
pub mod com_usart;
pub mod com_cdc;
