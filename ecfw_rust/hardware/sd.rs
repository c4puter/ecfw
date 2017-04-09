/*
 * The MIT License (MIT)
 * Copyright (c) 2016 Chris Pavlina
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

extern crate asf_sd_mmc;

use rustsys::mutex::Mutex;

#[allow(unused)]
pub static SD: Mutex<Sd> = Mutex::new(Sd::new(0));

pub struct Sd {
    slot: u8,
}

#[derive(Copy,Clone,Debug)]
#[repr(u8)]
pub enum SdError {
    Ok = 0,
    InitOngoing = 1,
    ErrNoCard = 2,
    ErrUnusable = 3,
    ErrSlot = 4,
    ErrComm = 5,
    ErrParam = 6,
    WriteProtected = 7
}

#[derive(Copy,Clone,Debug)]
#[repr(u8)]
pub enum CardType {
    Unknown = 0,
    Sd = 1,
    Mmc = 2,
    Sdio = 4,
    Hc = 8,
    SdCombo = 5,
}

#[derive(Copy,Clone,Debug)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum CardVersion {
    Unknown     = 0,
    V1_0        = 0x10, // SD 1.0, 1.01
    V1_2        = 0x12, // MMC 1.2
    V1_4        = 0x14, // MMC 1.4
    V1_10       = 0x1a, // SD 1.10
    V2_0        = 0x20, // SD 2.0
    V2_2        = 0x22, // MMC 2.2
    V3_0        = 0x30, // SD 3.0, MMC 3.0
    V4_0        = 0x40, // MMC 4.0
}

/// Initialize the entire SD/MMC system. This is not per card.
pub fn init() {
    unsafe {
        asf_sd_mmc::sd_mmc_init();
    }
}

impl Sd {

    pub const fn new(slot: u8) -> Sd
    {
        Sd {
            slot: slot,
        }
    }

    /// Check whether the card is ready, initializing
    pub fn check(&mut self) -> SdError {
        assert!(self.slot == 0);
        let ec = unsafe { asf_sd_mmc::sd_mmc_check(self.slot) };
        SdError::from_code(ec)
    }

    /// Get card type. Must be initialized.
    pub fn cardtype(&mut self) -> CardType {
        let code = unsafe { asf_sd_mmc::sd_mmc_get_type(self.slot) };
        CardType::from_code(code)
    }

    /// Get card version. Must be initialized.
    pub fn version(&mut self) -> CardVersion {
        let code = unsafe { asf_sd_mmc::sd_mmc_get_version(self.slot) };
        CardVersion::from_code(code)
    }

    /// Get the capacity in kB. Must be initialized.
    pub fn capacity(&mut self) -> u32 {
        unsafe{ asf_sd_mmc::sd_mmc_get_capacity(self.slot) }
    }

    /// Get whether the card is write-protected. Must be initialized.
    pub fn writeprotected(&mut self) -> bool {
        unsafe{ asf_sd_mmc::sd_mmc_is_write_protected(self.slot) }
    }
}

impl SdError {
    pub fn from_code(code: u8) -> SdError {
        match code as u32 {
            asf_sd_mmc::SD_MMC_OK           => SdError::Ok,
            asf_sd_mmc::SD_MMC_INIT_ONGOING => SdError::InitOngoing,
            asf_sd_mmc::SD_MMC_ERR_NO_CARD  => SdError::ErrNoCard,
            asf_sd_mmc::SD_MMC_ERR_UNUSABLE => SdError::ErrUnusable,
            asf_sd_mmc::SD_MMC_ERR_SLOT     => SdError::ErrSlot,
            asf_sd_mmc::SD_MMC_ERR_COMM     => SdError::ErrComm,
            asf_sd_mmc::SD_MMC_ERR_PARAM    => SdError::ErrParam,
            asf_sd_mmc::SD_MMC_ERR_WP       => SdError::WriteProtected,
            _ => SdError::ErrParam,
        }
    }
}

impl CardType {
    pub fn from_code(code: u8) -> CardType {
        match code as u32 {
            asf_sd_mmc::CARD_TYPE_SD        => CardType::Sd,
            asf_sd_mmc::CARD_TYPE_MMC       => CardType::Mmc,
            asf_sd_mmc::CARD_TYPE_SDIO      => CardType::Sdio,
            asf_sd_mmc::CARD_TYPE_HC        => CardType::Hc,
            asf_sd_mmc::CARD_TYPE_SD_COMBO  => CardType::SdCombo,
            _                               => CardType::Unknown,
        }
    }
}

impl CardVersion {
    pub fn from_code(code: u8) -> CardVersion {
        match code {
            0x10    => CardVersion::V1_0,
            0x12    => CardVersion::V1_2,
            0x14    => CardVersion::V1_4,
            0x1a    => CardVersion::V1_10,
            0x20    => CardVersion::V2_0,
            0x22    => CardVersion::V2_2,
            0x30    => CardVersion::V3_0,
            0x40    => CardVersion::V4_0,
            _       => CardVersion::Unknown,
        }
    }
}
