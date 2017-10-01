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

use os::{RwLock, Mutex};
use drivers::ledmatrix::LedMatrix;
use drivers::sd::Sd;
use drivers::tempsensor::TempSensor;
use devices::twi;

pub static MATRIX: RwLock<LedMatrix> = RwLock::new(LedMatrix::new(&twi::U801));

pub static SD: Mutex<Sd> = Mutex::new(Sd::new(0));

pub static SENSOR_LOGIC: TempSensor = TempSensor::new(&twi::LM75B_LOGIC);

pub static SENSOR_AMBIENT: TempSensor = TempSensor::new(&twi::LM75B_AMBIENT);
