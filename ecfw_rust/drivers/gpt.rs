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

//! GPT reader module

use data::utf;
use drivers::sd::*;
use messages::*;
use os::Mutex;
use core::fmt;
use core::str;

pub const BOOT_GUID: Guid = Guid::from_raw(
    0x7cca2c66, 0xb705, 0x58cb, 0xb9d9, 0xe16a166b84d9);

// GPT main header layout:      all values little endian
//
//      00 01 02 03 04 05 06 07   08 09 0a 0b 0c 0d 0e 0f
// 0x00 Signature______________   Revision___ Header size
// 0x10 CRC32______ Reserved___   Current LBA____________
// 0x20 Backup LBA_____________   First usable LBA_______
// 0x30 Last usable LBA________   Disk GUID______________
// 0x40 ____disk GUID (ctd)____   First LBA of entries___
// 0x50 Nb entries_ Entry size_   Array CRC32 Reserved___
//
// Signature is ASCII string "EFI PART"
// Revision expected to be 00 00 01 00
// Header size: 92 bytes
//
//
// Entry layout:
//
//      00 01 02 03 04 05 06 07   08 09 0a 0b 0c 0d 0e 0f
// 0x00 Partition type GUID______________________________
// 0x10 Unique partition GUID____________________________
// 0x20 First LBA______________   Last LBA_______________
// 0x30 Attributes_____________   Partition name (UTF16LE)
// 0x40 _________________________________________________
// 0x50 _________________________________________________
// 0x60 _________________________________________________
// 0x70 _________________________________________________
//
// GUID layout:
// GUIDs are stored in blocks, with the first three little-
// endian and the last two big-endian...
//
//      00 01 02 03 04 05 06 07   08 09 0a 0b 0c 0d 0e 0f
// 0x00 a__________ b____ c____   d____ e_______________

const GPT_HEADER_LBA: usize = 1;
const BLOCK_SIZE: usize = 512;
const SIGNATURE: u64 = 0x4546492050415254;  // "EFI PART"

/// GPT header.
pub struct Gpt<'a> {
    buffer: [u8; BLOCK_SIZE],
    iblock: usize,
    guid: Guid,
    entry_len: usize,
    lba_entries: usize,
    number_entries: usize,
    sd: &'a Mutex<Sd>,
}

impl<'a> Gpt<'a> {
    pub const fn new(sd: &Mutex<Sd>) -> Gpt
    {
        Gpt {
            buffer: [0u8; BLOCK_SIZE],
            iblock: 0,
            guid: Guid::new(),
            entry_len: 0,
            lba_entries: 0,
            number_entries: 0,
            sd: sd,
        }
    }

    /// Read the initial GPT header from the card
    pub fn read_header(&mut self) -> StdResult
    {
        self.buffer_block(GPT_HEADER_LBA)?;

        let sig = read_be(&self.buffer[0..8]);
        if sig != SIGNATURE {
            return Err(ERR_GPT_SIGNATURE);
        }

        self.entry_len = read_le(&self.buffer[0x54..0x58]) as usize;
        if self.entry_len == 0 {
            return Err(ERR_GPT_ZEROLEN);
        }
        if BLOCK_SIZE % self.entry_len != 0 {
            return Err(ERR_GPT_SIZEMULT);
        }

        self.lba_entries = read_le(&self.buffer[0x48..0x50]) as usize;
        self.number_entries = read_le(&self.buffer[0x50..0x54]) as usize;

        self.guid = Guid::from_bytes(&self.buffer[0x38..0x48]);

        Ok(())
    }

    /// Return the disk's GUID. Must be initialized.
    pub fn guid(&self) -> &Guid { &self.guid }

    /// Get the total number of partition entries. Note that this is not the
    /// number of *used* entries; you must read them to know if they're used.
    pub fn number_entries(&self) -> usize { self.number_entries }

    /// Populate a GptEntry structure with a given entry.
    pub fn read_entry(&mut self, ientry: usize, gptentry: &mut GptEntry)
        -> StdResult
    {
        let entries_per_block = BLOCK_SIZE / self.entry_len;
        let block_index = ientry / entries_per_block + self.lba_entries;
        let block_offset = self.entry_len * (ientry % entries_per_block);
        let entry_end = block_offset + self.entry_len;

        self.buffer_block(block_index)?;
        gptentry.load(&self.buffer[block_offset..entry_end])?;
        Ok(())
    }

    /// Populate a GptEntry structure with the boot partition. If no boot
    /// partition is found, returns success but the entry will be invalid.
    /// If multiple boot partitions are found, returns the first.
    pub fn read_boot(&mut self, gptentry: &mut GptEntry) -> StdResult
    {
        for i in 0..self.number_entries() {
            self.read_entry(i, gptentry)?;

            if gptentry.valid() {
                if gptentry.type_guid == BOOT_GUID {
                    return Ok(())
                } else {
                    gptentry.clear();
                }
            }
        }

        Ok(())
    }

    /// Read a block into the buffer, unless it's currently in the buffer.
    fn buffer_block(&mut self, iblock: usize) -> StdResult
    {
        if self.iblock == iblock {
            Ok(())
        } else {
            match self.sd.lock().read_block(iblock, &mut self.buffer) {
                Ok(()) => {
                    self.iblock = iblock;
                    Ok(()) },
                Err(e) => {
                    self.iblock = 0;    // buffer is invalid
                    Err(e) },
            }
        }
    }
}

#[derive(Copy,Clone,PartialEq)]
pub struct Guid {
    a: u32,
    b: u16,
    c: u16,
    d: u16,
    e: u64,
}

impl Guid {
    pub const fn new() -> Guid
    {
        Guid {
            a: 0u32,
            b: 0u16,
            c: 0u16,
            d: 0u16,
            e: 0u64,
        }
    }

    pub const fn from_raw(a: u32, b: u16, c: u16, d: u16, e: u64) -> Guid
    {
        Guid {
            a: a, b: b, c: c, d: d, e: e & 0xffffffffffff
        }
    }

    /// Read a GUID from bytes as stored in GPT.
    /// data must be 16 bytes long.
    pub fn from_bytes(data: &[u8]) -> Guid
    {
        assert!(data.len() == 16);
        Guid {
            a: read_le(&data[0..4]) as u32,
            b: read_le(&data[4..6]) as u16,
            c: read_le(&data[6..8]) as u16,
            d: read_be(&data[8..10]) as u16,
            e: read_be(&data[10..16])
        }
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
               self.a, self.b, self.c, self.d, self.e)
    }
}

pub struct GptEntry {
    pub type_guid: Guid,
    pub part_guid: Guid,
    pub start_lba: usize,
    pub end_lba: usize,
    pub attributes: u32,
    name_buf: [u8; /* codepoints */ 72 * /* UTF8 max per cp */ 4],
    name_len: usize,
}

impl GptEntry {

    /// Return a new, empty entry.
    pub const fn new() -> GptEntry {
        GptEntry {
            type_guid: Guid::new(),
            part_guid: Guid::new(),
            start_lba: 0,
            end_lba: 0,
            attributes: 0,
            name_buf: [0u8; 72*4],
            name_len: 0,
        }
    }

    /// Load a GPT entry given a block of raw data
    pub fn load(&mut self, data: &[u8]) -> StdResult
    {
        self.type_guid = Guid::from_bytes(&data[0x00..0x10]);
        self.part_guid = Guid::from_bytes(&data[0x10..0x20]);
        self.start_lba = read_le(&data[0x20..0x28]) as usize;
        self.end_lba = read_le(&data[0x28..0x30]) as usize;
        self.attributes = read_le(&data[0x30..0x38]) as u32;

        self.name_len =
            utf::read_utf16le_into_utf8(&mut self.name_buf, &data[0x38..data.len()])?;
        Ok(())
    }

    /// Clear a GPT entry so .valid() returns false
    pub fn clear(&mut self)
    {
        self.start_lba = 0
    }

    pub fn name(&self) -> &str
    {
        unsafe{str::from_utf8_unchecked(&self.name_buf[0..self.name_len])}
    }

    pub fn valid(&self) -> bool
    {
        self.start_lba > GPT_HEADER_LBA
    }
}

/// Read up to eight bytes in little endian.
fn read_be(data: &[u8]) -> u64
{
    let mut out = 0u64;

    for i in data {
        out <<= 8;
        out |= *i as u64;
    }

    out
}

/// Read up to eight bytes in big endian.
fn read_le(data: &[u8]) -> u64
{
    read_be(data).swap_bytes() >> (8 * (8 - data.len()))
}

