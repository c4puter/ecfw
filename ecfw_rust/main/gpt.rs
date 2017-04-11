/*
 * The MIT License (MIT)
 * Copyright (c) 2017 Chris Pavlina
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
 * OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
 * OR OTHER DEALINGS IN THE SOFTWARE.
 */

//! GPT reader module

use hardware::sd::*;
#[allow(unused)] use rustsys::ec_io;
use core::fmt;
use core::str;
use core::char;

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
pub struct Gpt {
    buffer: [u8; BLOCK_SIZE],
    iblock: usize,
    guid: Guid,
    entry_len: usize,
    lba_entries: usize,
    number_entries: usize,
}

impl Gpt {
    pub const fn new() -> Gpt
    {
        Gpt {
            buffer: [0u8; BLOCK_SIZE],
            iblock: 0,
            guid: Guid::new(),
            entry_len: 0,
            lba_entries: 0,
            number_entries: 0,
        }
    }

    /// Read the initial GPT header from the card
    pub fn read_header(&mut self) -> Result<(),&'static str>
    {
        try!(self.buffer_block(GPT_HEADER_LBA));

        let sig = read_be(&self.buffer[0..8]);
        if sig != SIGNATURE {
            return Err("invalid GPT signature");
        }

        self.entry_len = read_le(&self.buffer[0x54..0x58]) as usize;
        if self.entry_len == 0 {
            return Err("zero entry length");
        }
        if BLOCK_SIZE % self.entry_len != 0 {
            return Err("block size must be multiple of GPT entry length");
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
        -> Result<(), &'static str>
    {
        let entries_per_block = BLOCK_SIZE / self.entry_len;
        let block_index = ientry / entries_per_block + self.lba_entries;
        let block_offset = self.entry_len * (ientry % entries_per_block);
        let entry_end = block_offset + self.entry_len;

        try!(self.buffer_block(block_index));
        try!(gptentry.load(&self.buffer[block_offset..entry_end]));
        Ok(())
    }

    /// Read a block into the buffer, unless it's currently in the buffer.
    fn buffer_block(&mut self, iblock: usize) -> Result<(), &'static str>
    {
        if self.iblock == iblock {
            Ok(())
        } else {
            let mut sd = SD.lock();
            match sd.read_block(iblock, &mut self.buffer) {
                SdError::Ok => {
                    self.iblock = iblock;
                    Ok(()) },
                _ => {
                    self.iblock = 0;    // buffer is invalid
                    Err("SD error") }
            }
        }
    }
}

#[derive(Copy,Clone)]
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
    pub fn load(&mut self, data: &[u8]) -> Result<(), &'static str>
    {
        self.type_guid = Guid::from_bytes(&data[0x00..0x10]);
        self.part_guid = Guid::from_bytes(&data[0x10..0x20]);
        self.start_lba = read_le(&data[0x20..0x28]) as usize;
        self.end_lba = read_le(&data[0x28..0x30]) as usize;
        self.attributes = read_le(&data[0x30..0x38]) as u32;

        self.name_len = try!(
            read_utf16le_into_utf8(&mut self.name_buf, &data[0x38..data.len()]));
        Ok(())
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

/// Read UTF-16LE data into a UTF-8 buffer. Processes the entire length, but
/// returns the number of consecutive nonzero bytes written.
///
/// Errors on invalid codepoints, including orphaned surrogates. Asserts that
/// dest is long enough (must be at least src.len()*2).
///
/// Warning - this is not necessarily standard-compliant: it does not guarantee
/// errors on invalid conditions, only when it can't figure out what to do.
/// In particular, a single isolated high surrogate will be silently dropped
/// (whereas a single isolated low surrogate will cause complaint).
fn read_utf16le_into_utf8(dest: &mut [u8], src: &[u8]) -> Result<usize, &'static str>
{
    assert!(dest.len() >= src.len() * 2);

    let mut prev_surrogate = 0u32;
    let mut codepoint = 0u32;
    let mut idest = 0usize;
    let mut first_zero: Option<usize> = None;

    for isrc in 0..src.len() {
        if isrc % 2 == 0 {
            codepoint = src[isrc] as u32;
            continue;
        }

        codepoint |= (src[isrc] as u32) << 8;

        if codepoint >= 0xD800 && codepoint <= 0xDBFF {
            // high surrogate
            prev_surrogate = codepoint;
            continue;
        }
        if codepoint >= 0xDC00 && codepoint <= 0xDFFF {
            // low surrogate
            if prev_surrogate == 0 {
                return Err("orphaned UTF-16 surrogate");
            } else {
                codepoint = 0x10000 +
                    ((prev_surrogate & 0x03ff) << 10) +
                    (codepoint & 0x03ff);
            }
        }
        if codepoint > 0x10FFFF {
            return Err("invalid codepoint");
        }

        if codepoint == 0 && first_zero.is_none() {
            first_zero = Some(idest);
        }

        idest += write_one_utf8(dest, idest, codepoint);
    }

    match first_zero {
        Some(x) => { Ok(x) },
        None    => { Ok(idest) },
    }
}

/// Write a single codepoint into a buffer, returning the number of bytes written
fn write_one_utf8(dest: &mut [u8], idest: usize, codepoint: u32) -> usize
{
    let c = unsafe{char::from_u32_unchecked(codepoint)};
    let destlen = dest.len();
    let s = c.encode_utf8(&mut dest[idest..destlen]);
    s.len()
}
