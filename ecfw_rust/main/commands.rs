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

use os;
use drivers::{ext4, gpt};
use drivers::gpio::Gpio;
use drivers::ftrans::FTrans;
use devices;
use data::{ParseInt, hexprint};
use devices::pins::*;
use main::{reset, sysman};
use messages::*;
use core::fmt;
use alloc::string::String;

pub struct Command<'a> {
    pub name: &'a str,
    pub f: fn(args: &[&str]) -> StdResult,
    pub descr: &'a str,
}

pub static COMMAND_TABLE: &[Command] = &[
    Command{ name: "help",      f: cmd_help,    descr: "display commands and their descriptions" },
    Command{ name: "free",      f: cmd_free,    descr: "display free heap" },
    Command{ name: "reset",     f: cmd_reset,   descr: "reset entire system" },
    Command{ name: "dbgen",     f: cmd_dbgen,   descr: "enabled debug item" },
    Command{ name: "dbgdis",    f: cmd_dbgdis,  descr: "disable debug item" },
    Command{ name: "dbgls",     f: cmd_dbgls,   descr: "list debug items" },

    Command{ name: "panel",     f: cmd_panel,   descr: "render the user IO panel to the console" },
    Command{ name: "temps",     f: cmd_temps,   descr: "read the temperature sensors" },
    Command{ name: "event",     f: cmd_event,   descr: "send an event (boot, shutdown, reboot)" },

    Command{ name: "i2c_probe", f: cmd_i2c_probe,   descr: "probe I2C for an ADDR" },
    Command{ name: "i2c_read",  f: cmd_i2c_read,    descr: "read I2C from ADDR at LOCATION, N bytes" },
    Command{ name: "i2c_write", f: cmd_i2c_write,   descr: "write I2C to ADDR at LOCATION, BYTES" },

    Command{ name: "gpio_read", f: cmd_gpio_read,   descr: "read GPIO (by name)" },
    Command{ name: "gpio_write",f: cmd_gpio_write,  descr: "write to GPIO (by name) VALUE" },

    Command{ name: "clkdiv",    f: cmd_clkdiv,      descr: "set clock divider N to VALUE" },
    Command{ name: "clkrat",    f: cmd_clkrat,      descr: "set clock PLL ratio to N/M" },
    Command{ name: "clkload",   f: cmd_clkload,     descr: "set clock load capacitance to PF" },

    Command{ name: "pwr_stat",  f: cmd_pwr_stat,    descr: "display status of SUPPLY" },

    Command{ name: "mount",     f: cmd_mount,       descr: "mount SD card" },
    Command{ name: "umount",    f: cmd_umount,      descr: "unmount SD card" },
    Command{ name: "sync",      f: cmd_sync,        descr: "flush filesystem cache" },
    Command{ name: "sdinfo",    f: cmd_sdinfo,      descr: "print SD card info" },
    Command{ name: "readblock", f: cmd_readblock,   descr: "read block N from card" },
    Command{ name: "writeblock",f: cmd_writeblock,  descr: "write to block N, DATA..." },
    Command{ name: "partinfo",  f: cmd_partinfo,    descr: "dump GPT partition info" },
    Command{ name: "ls",        f: cmd_ls,          descr: "list PATH" },
    Command{ name: "hd",        f: cmd_hd,          descr: "hexdump the first block of PATH" },
    Command{ name: "bitstream", f: cmd_bitstream,   descr: "load fpga N with PATH" },
    Command{ name: "readlink",  f: cmd_readlink,    descr: "readlink" },
    Command{ name: "rm",        f: cmd_rm,          descr: "delete PATH" },
    Command{ name: "expand",    f: cmd_expand,      descr: "expand PATH, following links" },
    Command{ name: "ftrans",    f: cmd_ftrans,      descr: "open file transfer (requires USB)" },

    Command{ name: "peek",      f: cmd_peek,        descr: "read 32 bits at ADDR" },
    Command{ name: "poke",      f: cmd_poke,        descr: "write to ADDR, 32 bit DATA" },
];

fn argv_parsed<T, U>(
    args:   &[&str],
    n:      usize,
    _name:  &str,
    parser: fn(&str) -> Result<T, U>,
) -> Result<T, Error>
where
    U: fmt::Display,
{
    match parser(args[n]) {
        Ok(val) => Ok(val),
        Err(_) => Err(ERR_PARSE_ARGUMENT),
    }
}

fn cmd_help(_args: &[&str]) -> StdResult
{
    for i in 0 .. COMMAND_TABLE.len() {
        let ref cmd = COMMAND_TABLE[i];
        println!("{:12} - {}", cmd.name, cmd.descr);
    }

    Ok(())
}

fn cmd_free(_args: &[&str]) -> StdResult
{
    println!(
        "{} B, worst case {} B",
        os::freertos::get_free_heap(),
        os::freertos::get_worst_free_heap()
    );
    Ok(())
}

fn cmd_reset(_args: &[&str]) -> StdResult
{
    reset::hard_reset();
    Err(ERR_RESET_FAILED)
}

fn cmd_dbgen(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        Err(ERR_EXPECTED_ARGS)
    } else {
        if debug_set(args[1], true) {
            Ok(())
        } else {
            Err(ERR_CANNOT_FIND)
        }
    }
}

fn cmd_dbgdis(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        Err(ERR_EXPECTED_ARGS)
    } else {
        if debug_set(args[1], false) {
            Ok(())
        } else {
            Err(ERR_CANNOT_FIND)
        }
    }
}

fn cmd_dbgls(_args: &[&str]) -> StdResult
{
    for &dbg in DEBUG_TABLE {
        println!(
            "{}    {}",
            if dbg.enabled() { "en " } else { "dis" },
            dbg.name
        );
    }
    Ok(())
}

fn cmd_panel(_args: &[&str]) -> StdResult
{
    fn r_(v: bool) -> &'static str
    {
        match v {
            true => "R ",
            false => "  ",
        }
    }
    fn g_(v: bool) -> &'static str
    {
        match v {
            true => "G ",
            false => "  ",
        }
    }
    fn yn(v: bool) -> &'static str
    {
        match v {
            true => "Y ",
            false => " N",
        }
    }

    println!(
        "P12V   {} {} | P3V3_STBY  {} {} | EC FMW {} {}       {} {} STATE FAIL",
        r_(P12V_PCI_R.get()),
        g_(P12V_PCI_G.get()),
        r_(P3V3_STBY_R.get()),
        g_(P3V3_STBY_G.get()),
        r_(ECFW_R.get()),
        g_(ECFW_G.get()),
        r_(STATE_FAIL_R.get()),
        g_(STATE_FAIL_G.get())
    );
    println!(
        "P5V_A  {} {} | P3V3_AUX   {} {} | PWR SQ {} {}       {} {} UNC1",
        r_(P5V_PCI_A_R.get()),
        g_(P5V_PCI_A_G.get()),
        r_(P3V3_AUX_R.get()),
        g_(P3V3_AUX_G.get()),
        r_(POWER_R.get()),
        g_(POWER_G.get()),
        r_(UNC1_R.get()),
        g_(UNC1_G.get())
    );
    println!(
        "P5V_B  {} {} | P3V3_LOGIC {} {} | CARD   {} {}       {} {} UNC2",
        r_(P5V_PCI_B_R.get()),
        g_(P5V_PCI_B_G.get()),
        r_(P3V3_LOGIC_R.get()),
        g_(P3V3_LOGIC_G.get()),
        r_(CARD_R.get()),
        g_(CARD_G.get()),
        r_(UNC2_R.get()),
        g_(UNC2_G.get())
    );
    println!(
        "P3V3_A {} {} | P1V5_LOGIC {} {} | BITSTR {} {} {} {} {} {} UNC3",
        r_(P3V3_PCI_A_R.get()),
        g_(P3V3_PCI_A_G.get()),
        r_(P1V5_LOGIC_R.get()),
        g_(P1V5_LOGIC_G.get()),
        r_(BIT_R.get()),
        g_(BIT_BRIDGE_G.get()),
        g_(BIT_CPU0_G.get()),
        g_(BIT_CPU1_G.get()),
        r_(UNC3_R.get()),
        g_(UNC3_G.get())
    );
    println!(
        "P3V3_B {} {} | P1V2_LOGIC {} {} | MEM LD {} {}       {} {} UNC4",
        r_(P3V3_PCI_B_R.get()),
        g_(P3V3_PCI_B_G.get()),
        r_(P1V2_LOGIC_R.get()),
        g_(P1V2_LOGIC_G.get()),
        r_(MEM_R.get()),
        g_(MEM_G.get()),
        r_(UNC4_R.get()),
        g_(UNC4_G.get())
    );
    println!(
        "N12V   {} {} | PV75_TERM  {} {} | RUN    {} {}    {} {} {} UNC5",
        r_(N12V_PCI_R.get()),
        g_(N12V_PCI_G.get()),
        r_(PV75_TERM_R.get()),
        g_(PV75_TERM_G.get()),
        r_(RUN_R.get()),
        g_(RUN_G.get()),
        g_(UPDOG_G.get()),
        r_(UNC5_R.get()),
        g_(UNC5_G.get())
    );
    println!("");
    println!("{} UNC0", yn(UNC_SW_0.get()));
    println!("{} UNC1", yn(UNC_SW_1.get()));
    println!("{} UNC2", yn(UNC_SW_2.get()));
    println!("{} low speed", yn(LOW_SPEED.get()));
    println!("{} force pwr", yn(FORCE_POWER.get()));
    println!("{} single CPU", yn(SINGLE_CPU.get()));
    println!("{} debug boot", yn(DEBUG_BOOT.get()));
    println!("{} merged ser", yn(MERGE_SERIAL.get()));
    Ok(())
}

fn cmd_temps(_args: &[&str]) -> StdResult
{
    let temp_logic = devices::SENSOR_LOGIC.read()?;
    let temp_ambient = devices::SENSOR_AMBIENT.read()?;

    println!("Logic:   {}.{} degC", temp_logic / 10, temp_logic % 10);
    println!("Ambient: {}.{} degC", temp_ambient / 10, temp_ambient % 10);

    Ok(())
}

fn cmd_event(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        Err(ERR_EXPECTED_ARGS)
    } else if args[1] == "boot" {
        sysman::post(sysman::Event::Boot);
        Ok(())
    } else if args[1] == "shutdown" {
        sysman::post(sysman::Event::Shutdown);
        Ok(())
    } else if args[1] == "reboot" {
        sysman::post(sysman::Event::Reboot);
        Ok(())
    } else {
        Err(ERR_CANNOT_FIND)
    }
}

fn cmd_i2c_probe(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let addr = argv_parsed(args, 1, "ADDR", u8::parseint)?;

    let is_present = devices::twi::TWI0.probe(addr)?;

    if is_present {
        println!("address {} present", addr);
    } else {
        println!("address {} does not respond", addr);
    }
    Ok(())
}

fn cmd_i2c_read(args: &[&str]) -> StdResult
{
    if args.len() < 4 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let addr = argv_parsed(args, 1, "ADDR", u8::parseint)?;
    let loc = argv_parsed(args, 2, "LOCATION", u8::parseint)?;
    let n = argv_parsed(args, 3, "N", u8::parseint)?;
    if n > 16 {
        return Err(ERR_ARG_RANGE);
    }

    let location_arr = [loc];
    let mut buffer = [0 as u8; 16];

    devices::twi::TWI0.read(
        addr,
        &location_arr,
        &mut buffer[0 .. n as usize],
    )?;
    println!("{:?}", &buffer[0 .. n as usize]);
    Ok(())
}

fn cmd_i2c_write(args: &[&str]) -> StdResult
{
    if args.len() < 3 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let addr = argv_parsed(args, 1, "ADDR", u8::parseint)?;
    let loc = argv_parsed(args, 2, "LOCATION", u8::parseint)?;

    if args.len() > 19 {
        return Err(ERR_TOO_MANY_ARGS);
    }

    let mut buffer = [0 as u8; 16];
    let n = args.len() - 3;
    for i in 0 .. n {
        let arg = argv_parsed(args, i + 3, "BYTES", u8::parseint)?;
        buffer[i] = arg;
    }

    let location_arr = [loc];

    devices::twi::TWI0.write(
        addr,
        &location_arr,
        &buffer[0 .. n as usize],
    )?;
    Ok(())
}

fn cmd_gpio_read(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let gpio_name = args[1];

    match devices::pins::PIN_TABLE.iter().find(|&pin| {
        *(pin.name()) == *gpio_name
    }) {
        Some(pin) => println!("{}", pin.get()),
        None => println!("pin {} not found", gpio_name),
    }

    Ok(())
}

fn cmd_gpio_write(args: &[&str]) -> StdResult
{
    if args.len() < 3 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let gpio_name = args[1];
    let gpio_val = argv_parsed(args, 2, "VALUE", i8::parseint)?;

    match devices::pins::PIN_TABLE.iter().find(|&pin| {
        *(pin.name()) == *gpio_name
    }) {
        Some(pin) => pin.set(gpio_val != 0),
        None => println!("pin {} not found", gpio_name),
    }

    Ok(())
}

fn cmd_clkdiv(args: &[&str]) -> StdResult
{
    if args.len() < 3 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let n_divider = argv_parsed(args, 1, "N", u32::parseint)?;
    let div_val = argv_parsed(args, 2, "VALUE", u32::parseint)?;

    if n_divider == 1 {
        devices::CLOCK_SYNTH.y1div(div_val)
    } else if n_divider == 2 {
        devices::CLOCK_SYNTH.y2div(div_val)
    } else if n_divider == 3 {
        devices::CLOCK_SYNTH.y3div(div_val)
    } else {
        Err(ERR_ARG_RANGE)
    }
}

fn cmd_clkrat(args: &[&str]) -> StdResult
{
    if args.len() < 3 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let num = argv_parsed(args, 1, "N", u32::parseint)?;
    let den = argv_parsed(args, 2, "M", u32::parseint)?;

    devices::CLOCK_SYNTH.ratio(num, den)?;
    devices::CLOCK_SYNTH.usepll(true)?;
    Ok(())
}

fn cmd_clkload(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let load = argv_parsed(args, 1, "PF", u32::parseint)?;

    devices::CLOCK_SYNTH.loadcap(load)
}

fn cmd_pwr_stat(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let supply_name = args[1];
    let _lock = devices::supplies::POWER_MUTEX.lock();
    let mut it = devices::supplies::SUPPLY_TABLE.iter();
    match it.find(|&supply| *(supply.name()) == *supply_name) {
        Some(supply) => {
            println!("supply {} status: {:?}", supply_name, supply.status()?)
        },
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}

fn cmd_mount(_args: &[&str]) -> StdResult
{
    if !CARD.get() {
        return Err(ERR_NO_CARD);
    }

    CARDEN.set(true);
    os::delay(1);
    devices::SD.lock().check()?;

    let mut table = gpt::Gpt::new(&devices::SD);
    let mut entry = gpt::GptEntry::new();

    table.read_header()?;
    table.read_boot(&mut entry)?;

    if !entry.valid() {
        return Err(ERR_NO_BOOT_PART);
    }

    let bd = ext4::makedev(&devices::SD, &entry);
    ext4::register_device(bd, "root")?;

    ext4::mount("root", "/", false)?;
    Ok(())
}

fn cmd_umount(_args: &[&str]) -> StdResult
{
    ext4::umount("/")?;
    ext4::unregister_device("root")?;

    if !CARD.get() {
        return Err(ERR_NO_CARD);
    }
    CARDEN.set(false);
    Ok(())
}

fn cmd_sync(_args: &[&str]) -> StdResult
{
    ext4::sync("/")
}

fn cmd_sdinfo(_args: &[&str]) -> StdResult
{
    if !CARD.get() {
        return Err(ERR_NO_CARD);
    }

    let mut sd = devices::SD.lock();

    println!("Type:      {:?}", sd.cardtype());
    println!("Version:   {:?}", sd.version());
    println!("Capacity:  {:?} MiB", sd.capacity() / 1024);
    println!(
        "Protected: {}",
        if sd.writeprotected() { "yes" } else { "no" }
    );

    Ok(())
}

fn cmd_readblock(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let iblock = argv_parsed(args, 1, "BLOCK", u32::parseint)? as usize;

    let mut buf = [0u8; 512];
    devices::SD.lock().read_block(iblock, &mut buf)?;

    hexprint(&buf);
    Ok(())
}

fn cmd_writeblock(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let iblock = argv_parsed(args, 1, "BLOCK", u32::parseint)? as usize;

    let mut buf = [0u8; 512];
    for i in 2 .. args.len() {
        let data = argv_parsed(args, i, "DATA", u8::parseint)?;
        buf[i - 2] = data;
    }

    devices::SD.lock().write_block(iblock, &buf)?;
    Ok(())
}

fn cmd_partinfo(_args: &[&str]) -> StdResult
{
    let mut table = gpt::Gpt::new(&devices::SD);
    let mut entry = gpt::GptEntry::new();

    table.read_header()?;

    println!("Disk GUID: {}", table.guid());

    for i in 0 .. table.number_entries() {
        table.read_entry(i, &mut entry)?;
        if !entry.valid() {
            continue;
        }

        println!("Entry {}:", i);
        println!("  Type GUID:   {}", entry.type_guid);
        println!("  Unique GUID: {}", entry.part_guid);
        println!(
            "  Range:       {:08x}...{:08x}",
            entry.start_lba,
            entry.end_lba
        );
        println!("  Attributes:  {:08x}", entry.attributes);
        println!("  Name:        {}", entry.name());
    }

    Ok(())
}

fn cmd_ls(args: &[&str]) -> StdResult
{
    let path = if args.len() == 2 { args[1] } else { "/" };

    // Use a stringbuilder to append each item to the path, for stat()
    let mut s = String::with_capacity(1024);
    s.push_str(path);
    let pab = path.as_bytes();
    if pab[pab.len() - 1] != '/' as u8 {
        s.push_str("/");
    }
    let only_path = s.len();

    let mut dir = ext4::dir_open(path)?;
    for de in dir.iter() {
        let name = de.name()?;
        s.push_str(name);
        let stat = ext4::stat(&s)?;
        s.truncate(only_path);

        println!("{} {:8} {}", stat, stat.size(), name);
    }
    Ok(())
}

fn cmd_hd(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[1];

    let mut file = ext4::fopen_expand(path, ext4::OpenFlags::Read)?;
    let mut buf = [0u8; 512];

    let bytes = file.read(&mut buf)?;

    hexprint(&buf[0 .. bytes]);
    Ok(())
}

fn cmd_bitstream(args: &[&str]) -> StdResult
{
    if args.len() < 3 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[2];
    let nfpga = argv_parsed(args, 1, "N", u32::parseint)? as usize;

    if nfpga >= devices::FPGAS.len() {
        return Err(ERR_ARG_RANGE);
    }

    devices::FPGAS[nfpga].load(path)
}

fn cmd_readlink(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[1];

    let link = ext4::readlink(path)?;

    println!("{}", link);
    Ok(())
}

fn cmd_rm(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[1];

    ext4::unlink(path)
}

fn cmd_expand(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[1];

    let path = ext4::expand(path)?;

    println!("{}", path);
    Ok(())
}

fn cmd_ftrans(_args: &[&str]) -> StdResult
{
    let mut ftrans = FTrans::new();
    ftrans.run();
    Ok(())
}

fn cmd_peek(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let addr = argv_parsed(args, 1, "ADDR", u64::parseint)?;
    let mut buf = [0u32];

    devices::NORTHBRIDGE.peek(&mut buf, addr)?;

    println!("{:08X}", buf[0]);

    Ok(())
}

fn cmd_poke(args: &[&str]) -> StdResult
{
    if args.len() < 3 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let addr = argv_parsed(args, 1, "ADDR", u64::parseint)?;
    let data = argv_parsed(args, 1, "DATA", u32::parseint)?;

    devices::NORTHBRIDGE.poke(addr, &[data])?;

    Ok(())
}
