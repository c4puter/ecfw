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

/*!
Static mutex. Lighter than FreeRTOS's static mutex and requires no
initialization.
*/

use os::{RwLock, RwLockWriter};

pub struct Mutex<T: Sized + Sync> {
    rwlock: RwLock<T>,
}

pub type MutexLock<'a, T> = RwLockWriter<'a, T>;

impl<T> Mutex<T> where T: Sized + Sync {
    pub const fn new(data: T) -> Mutex<T> {
        Mutex {rwlock: RwLock::new(data)}
    }

    pub fn lock(&self) -> MutexLock<T> {
        self.rwlock.write()
    }

    pub fn try_lock(&self) -> Option<MutexLock<T>> {
        self.rwlock.try_write()
    }

    pub fn lock_timeout(&self, nticks: u32) -> Option<MutexLock<T>> {
        self.rwlock.write_timeout(nticks)
    }

}
