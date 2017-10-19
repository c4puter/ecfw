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

extern crate asf_sd_mmc;
extern crate ctypes;

use messages::*;

pub struct Sd {
    slot: u8,
}

fn to_stdresult(code: u8) -> StdResult
{
    match code as u32 {
        asf_sd_mmc::SD_MMC_OK           => Ok(()),
        asf_sd_mmc::SD_MMC_INIT_ONGOING => Err(ERR_SD_INIT_ONGOING),
        asf_sd_mmc::SD_MMC_ERR_NO_CARD  => Err(ERR_NO_CARD),
        asf_sd_mmc::SD_MMC_ERR_UNUSABLE => Err(ERR_SD_UNUSABLE),
        asf_sd_mmc::SD_MMC_ERR_SLOT     => Err(ERR_SD_SLOT),
        asf_sd_mmc::SD_MMC_ERR_COMM     => Err(ERR_SD_COMM),
        asf_sd_mmc::SD_MMC_ERR_PARAM    => Err(ERR_SD_PARAM),
        asf_sd_mmc::SD_MMC_ERR_WP       => Err(ERR_SD_WRITE_PROT),
        _                               => Err(ERR_UNKNOWN)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum CardType {
    Unknown = 0,
    Sd = 1,
    Mmc = 2,
    Sdio = 4,
    Hc = 8,
    SdCombo = 5,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum CardVersion {
    Unknown = 0,
    V1_0 = 0x10, // SD 1.0, 1.01
    V1_2 = 0x12, // MMC 1.2
    V1_4 = 0x14, // MMC 1.4
    V1_10 = 0x1a, // SD 1.10
    V2_0 = 0x20, // SD 2.0
    V2_2 = 0x22, // MMC 2.2
    V3_0 = 0x30, // SD 3.0, MMC 3.0
    V4_0 = 0x40, // MMC 4.0
}

/// Initialize the entire SD/MMC system. This is not per card.
pub fn init()
{
    unsafe {
        asf_sd_mmc::sd_mmc_init();
    }
}

impl Sd {
    pub const fn new(slot: u8) -> Sd
    {
        Sd { slot: slot }
    }

    /// Check whether the card is ready, initializing
    pub fn check(&mut self) -> StdResult
    {
        assert!(self.slot == 0);

        // ASF SD driver will return SD_MMC_ERR_NO_CARD once, then
        // SD_MMC_INIT_ONGOING, then success. Call several times to
        // hide this stupid behavior.

        let mut got_no_card = false;

        loop {
            let ec = unsafe { asf_sd_mmc::sd_mmc_check(self.slot) };

            if ec as u32 == asf_sd_mmc::SD_MMC_ERR_NO_CARD && !got_no_card {
                got_no_card = true;
            } else if ec as u32 != asf_sd_mmc::SD_MMC_INIT_ONGOING {
                return to_stdresult(ec);
            }
        }
    }

    /// Get card type. Must be initialized.
    pub fn cardtype(&mut self) -> CardType
    {
        let code = unsafe { asf_sd_mmc::sd_mmc_get_type(self.slot) };
        CardType::from_code(code)
    }

    /// Get card version. Must be initialized.
    pub fn version(&mut self) -> CardVersion
    {
        let code = unsafe { asf_sd_mmc::sd_mmc_get_version(self.slot) };
        CardVersion::from_code(code)
    }

    /// Get the capacity in kB. Must be initialized.
    pub fn capacity(&mut self) -> u32
    {
        unsafe { asf_sd_mmc::sd_mmc_get_capacity(self.slot) }
    }

    /// Get whether the card is write-protected. Must be initialized.
    pub fn writeprotected(&mut self) -> bool
    {
        unsafe { asf_sd_mmc::sd_mmc_is_write_protected(self.slot) }
    }

    /// Read a block from the card. Blocks are 512B long. Must be
    /// initialized.
    pub fn read_block(
        &mut self,
        iblock: usize,
        dest: &mut [u8; 512],
    ) -> StdResult
    {
        unsafe { self.read_blocks(iblock, 1, dest.as_mut_ptr()) }
    }

    /// Write a block to the card. Blocks are 512B long. Must be
    /// initialized.
    pub fn write_block(&mut self, iblock: usize, src: &[u8; 512]) -> StdResult
    {
        unsafe { self.write_blocks(iblock, 1, src.as_ptr()) }
    }

    /// Read an arbitrary number of blocks into a pointer. Unsafe, intended
    /// for C interaction.
    pub unsafe fn read_blocks(
        &mut self,
        iblock: usize,
        nblocks: u16,
        dest: *mut u8,
    ) -> StdResult
    {
        to_stdresult(asf_sd_mmc::sd_mmc_init_read_blocks(
            self.slot,
            iblock as u32,
            nblocks,
        ))?;

        to_stdresult(asf_sd_mmc::sd_mmc_start_read_blocks(
            dest as *mut ctypes::c_void,
            nblocks,
        ))?;

        to_stdresult(asf_sd_mmc::sd_mmc_wait_end_of_read_blocks(false))?;

        Ok(())
    }

    /// Write an arbitrary number of blocks from a pointer. Unsafe, intended
    /// for C interaction.
    pub unsafe fn write_blocks(
        &mut self,
        iblock: usize,
        nblocks: u16,
        src: *const u8,
    ) -> StdResult
    {
        to_stdresult(asf_sd_mmc::sd_mmc_init_write_blocks(
            self.slot,
            iblock as u32,
            nblocks,
        ))?;

        to_stdresult(asf_sd_mmc::sd_mmc_start_write_blocks(
            src as *const ctypes::c_void,
            nblocks,
        ))?;

        to_stdresult(asf_sd_mmc::sd_mmc_wait_end_of_write_blocks(false))?;

        Ok(())
    }
}

impl CardType {
    pub fn from_code(code: u8) -> CardType
    {
        match code as u32 {
            asf_sd_mmc::CARD_TYPE_SD => CardType::Sd,
            asf_sd_mmc::CARD_TYPE_MMC => CardType::Mmc,
            asf_sd_mmc::CARD_TYPE_SDIO => CardType::Sdio,
            asf_sd_mmc::CARD_TYPE_HC => CardType::Hc,
            asf_sd_mmc::CARD_TYPE_SD_COMBO => CardType::SdCombo,
            _ => CardType::Unknown,
        }
    }
}

impl CardVersion {
    pub fn from_code(code: u8) -> CardVersion
    {
        match code {
            0x10 => CardVersion::V1_0,
            0x12 => CardVersion::V1_2,
            0x14 => CardVersion::V1_4,
            0x1a => CardVersion::V1_10,
            0x20 => CardVersion::V2_0,
            0x22 => CardVersion::V2_2,
            0x30 => CardVersion::V3_0,
            0x40 => CardVersion::V4_0,
            _ => CardVersion::Unknown,
        }
    }
}
