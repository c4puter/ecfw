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

//! Driver to initialize SDRAM.
use messages::*;
use os::freertos;
use devices::NORTHBRIDGE;
use devices::i2c::I2C0;
use devices::pins;
use drivers::gpio::Gpio;
use alloc::vec;
use core::str;

// drac_ddr3 has an "init" mode where address bits map to init controls as
// follows.
//
// The DRAC is mapped at base address 0xE00000000.
//
// INIT CONTROL     DRAC ADDRESS BITS       BRIDGE ADDRESS BITS
// BankAddr         sa[15:13]               wb_adr[13:11]
// Addr             sa[31:16]               wb_adr[29:14]
// CS               sa[32]                  wb_adr[30]
// Cmd              sa[10:8]                wb_adr[8:6]
// More Init        sa[33]                  wb_adr[31]
//
// The DRAC starts in this mode, and expects to receive bus writes to these
// "magic" addresses to perform initialization. When a write is received with
// sa[33] = 0, it will proceed into normal operating mode.
//
// Beware: while in init mode, no reads can be performed, and with the current
// bridge implementation will lock up the system bus.

const DRAC_BASE: u64 = 0xE_0000_0000;
const COMMAND_MRS: u64 = (0 << 6);
const COMMAND_NOP: u64 = (7 << 6);
const COMMAND_ZQCL: u64 = (6 << 6) | (1 << 24);
const COMMAND_CS1: u64 = 0;
const COMMAND_CS2: u64 = (1 << 30);
const INIT_CONTINUE: u64 = (1 << 31);

const SPD_I2C_ADDR: u8 = 0x50;

fn cmd_nop(cs: u64) -> StdResult
{
    NORTHBRIDGE.poke(DRAC_BASE | cs | COMMAND_NOP | INIT_CONTINUE, &[0xffffffffu32])
}

fn cmd_done() -> StdResult
{
    NORTHBRIDGE.poke(DRAC_BASE | COMMAND_NOP, &[0xffffffffu32])
}

fn cmd_zqcl(cs: u64) -> StdResult
{
    NORTHBRIDGE.poke(DRAC_BASE | cs | COMMAND_ZQCL | INIT_CONTINUE, &[0xffffffffu32])
}

fn cmd_mrs(cs: u64, reg: u32, value: u32) -> StdResult
{
    NORTHBRIDGE.poke(
        DRAC_BASE | cs | COMMAND_MRS | INIT_CONTINUE |
            ((value as u64) << 14) | ((reg as u64) << 11),
        &[0xffffffffu32])
}

fn calculate_mr0() -> u32
{
    //precharge PD: DLL on
    //|   write recovery 5
    //|   | DLL reset
    //|   | | reserved
    //|   | | |   CAS latency (5)
    //|   | | |   | READ burst type (sequential)
    //|   | | |   | | reserved
    //|   | | |   | | | burst length BC4 (chop)
    0b1_001_1_0_001_0_0_10
}

fn calculate_mr1() -> u32
{
    //reserved
    //| Q Off enabled
    //| | TDQS disabled
    //| | | reserved
    //| | | | Rtt high bit (001: 60/60 ohm)
    //| | | | | reserved
    //| | | | | | Write leveling disabled
    //| | | | | | | Rtt mid bit (001: 60/60 ohm)
    //| | | | | | | | ODS high bit (00: 40 ohm)
    //| | | | | | | | |  Additive latency disabled
    //| | | | | | | | |  | Rtt low bit (001: 60/60 ohm)
    //| | | | | | | | |  | | ODS low bit (00: 40 ohm)
    //| | | | | | | | |  | | | DLL enabled
    0b0_0_0_0_0_0_0_0_0_00_1_0_0
}

fn calculate_mr2() -> u32
{
    //  reserved
    //  |  Dynamic ODT RZQ/4
    //  |  | reserved
    //  |  | | Self refresh temperature 0-85C
    //  |  | | | Auto self refresh disabled
    //  |  | | | |   CAS write latency = 5
    //  |  | | | |   |   reserved
    0b000_01_0_0_0_000_000
}

fn calculate_mr3() -> u32
{
    //MPR enable: normal DRAM operations
    //|  MPR READ function: predefined pattern
    0b0_00
}

fn sdram_init_one(cs: u64) -> StdResult
{
    // Steps from Micron TN-41-07

    // 1. Ramp Vdd and Vddq, asserting RESET
    //      Done by power sequence.
    // 3. Apply Vtt and Vref
    // 4. Continue to assert RESET for at least 200us
    // 5. CKE must be LOW at least 10ns prior to step 6
    // 6. Deassert RESET
    // 7. Hold CKE LOW for at least 500us
    // 8. Assert NOP or DES
    cmd_nop(cs)?;

    // 9. Apply stable clocks
    // 10. Drive ODT LOW or HIGH
    // 11. Wait at least 10ns + 5 clocks
    // 12. Bring CKE HIGH
    // 13. Wait at least tXPR
    freertos::delay(1);

    // 14. Issue MRS to MR2
    cmd_mrs(cs, 2, calculate_mr2())?;

    // 15. Wait at least tMRD
    // 16. Issue MRS to MR3
    cmd_mrs(cs, 3, calculate_mr3())?;

    // 17. Wait at least tMRD
    // 18. Issue MRS to MR1
    cmd_mrs(cs, 1, calculate_mr1())?;

    // 19. Wait at least tMRD
    // 20. Issue MRS to MR0
    cmd_mrs(cs, 0, calculate_mr0())?;
    // 21. Wait at least tMOD
    freertos::delay(1);
    // 22. Issue ZQCL to calibrate Rtt and Ron
    cmd_zqcl(cs)?;
    freertos::delay(1);

    Ok(())
}

fn get_spd() -> Result<vec::Vec<u8>, Error>
{
    let mut buf = vec::from_elem(0u8, 128);

    I2C0.read(SPD_I2C_ADDR, &[0u8], &mut buf[0..128])?;

    Ok(buf)
}

fn spd_info_check(spd: &vec::Vec<u8>) -> StdResult
{
    let density = spd[4];
    debug!(DEBUG_SDRAM, "found SDRAM with capacity {}",
           (match density {
               1 => "2 GiB",
               2 => "4 GiB",
               3 => "8 GiB",
               4 => "16 GiB",
               _ => "(unknown)" }));

    let cas_support = spd[14] as u32 | ((spd[15] as u32) << 8);

    if cas_support & 0x2 != 0 {
        debug!(DEBUG_SDRAM, "CL=5 supported");
    } else {
        debug!(DEBUG_SDRAM, "CL=5 unsupported (DRAC has hardcoded CL)");
        return Err(ERR_CAS);
    }

    Ok(())
}

pub fn sdram_init() -> StdResult
{
    let spd = get_spd()?;
    spd_info_check(&spd)?;
    pins::SDRAM_RST.set(true);
    freertos::delay(1);
    pins::SDRAM_RST.set(false);
    sdram_init_one(COMMAND_CS1)?;
    sdram_init_one(COMMAND_CS2)?;
    cmd_done()?;
    Ok(())
}
