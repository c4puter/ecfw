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

use os;
use drivers;
use drivers::gpio::Gpio;
use drivers::ftrans::FTrans;
use devices;
use data::{ParseInt, hexprint, StringBuilder};
use devices::pins::*;
use main::{reset, sysman};
use bindgen_mcu;
use messages::*;
use core::fmt;
use alloc::vec;

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
    Command{ name: "extclk",    f: cmd_extclk,      descr: "use external clock" },
    Command{ name: "intclk",    f: cmd_intclk,      descr: "use internal clock" },

    Command{ name: "pwr_stat",  f: cmd_pwr_stat,    descr: "display status of SUPPLY" },

    Command{ name: "mount",     f: cmd_mount,       descr: "mount SD card" },
    Command{ name: "umount",    f: cmd_umount,      descr: "unmount SD card" },
    Command{ name: "sdinfo",    f: cmd_sdinfo,      descr: "print SD card info" },
    Command{ name: "readblock", f: cmd_readblock,   descr: "read block N from card" },
    Command{ name: "writeblock",f: cmd_writeblock,  descr: "write to block N, DATA..." },
    Command{ name: "partinfo",  f: cmd_partinfo,    descr: "dump GPT partition info" },
    Command{ name: "ls",        f: cmd_ls,          descr: "list PATH" },
    Command{ name: "hd",        f: cmd_hd,          descr: "hexdump the first block of PATH" },
    Command{ name: "spi_dump",  f: cmd_spi_dump,    descr: "dump PATH to SPI" },
    Command{ name: "readlink",  f: cmd_readlink,    descr: "readlink" },
    Command{ name: "expand",    f: cmd_expand,      descr: "expand PATH, following links" },
    Command{ name: "ftrans",    f: cmd_ftrans,      descr: "open file transfer" },
];

fn argv_parsed<T, U>(args: &[&str], n: usize, _name: &str, parser: fn(&str)->Result<T,U>) -> Result<T, Error>
    where U: fmt::Display
{
    match parser(args[n]) {
        Ok(val) => Ok(val),
        Err(_) => Err(ERR_PARSE_ARGUMENT)
    }
}

fn cmd_help(_args: &[&str]) -> StdResult
{
    for i in 0..COMMAND_TABLE.len() {
        let ref cmd = COMMAND_TABLE[i];
        println!("{:12} - {}", cmd.name, cmd.descr);
    }

    Ok(())
}

fn cmd_free(_args: &[&str]) -> StdResult
{
    println!("{} B, worst case {} B",
             os::freertos::get_free_heap(), os::freertos::get_worst_free_heap());
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
        println!("{}    {}",
            if dbg.enabled() { "en " } else { "dis" },
            dbg.name );
    }
    Ok(())
}

fn cmd_panel(_args: &[&str]) -> StdResult
{
    fn r_(v: bool) -> &'static str {
        match v { true => "R ", false => "  " }
    }
    fn g_(v: bool) -> &'static str {
        match v { true => "G ", false => "  " }
    }
    fn yn(v: bool) -> &'static str {
        match v { true => "Y ", false => " N" }
    }

    println!("P12V   {} {} | P3V3_STBY  {} {} | EC FMW {} {}       {} {} UNC0",
             r_(P12V_PCI_R.get()), g_(P12V_PCI_G.get()),
             r_(P3V3_STBY_R.get()), g_(P3V3_STBY_G.get()),
             r_(ECFW_R.get()), g_(ECFW_G.get()),
             r_(UNC0_R.get()), g_(UNC0_G.get()));
    println!("P5V_A  {} {} | P3V3_AUX   {} {} | PWR SQ {} {}       {} {} UNC1",
             r_(P5V_PCI_A_R.get()), g_(P5V_PCI_A_G.get()),
             r_(P3V3_AUX_R.get()), g_(P3V3_AUX_G.get()),
             r_(POWER_R.get()), g_(POWER_G.get()),
             r_(UNC1_R.get()), g_(UNC1_G.get()));
    println!("P5V_B  {} {} | P3V3_LOGIC {} {} | CARD   {} {}       {} {} UNC2",
             r_(P5V_PCI_B_R.get()), g_(P5V_PCI_B_G.get()),
             r_(P3V3_LOGIC_R.get()), g_(P3V3_LOGIC_G.get()),
             r_(CARD_R.get()), g_(CARD_G.get()),
             r_(UNC2_R.get()), g_(UNC2_G.get()));
    println!("P3V3_A {} {} | P1V5_LOGIC {} {} | BITSTR {} {} {} {} {} {} UNC3",
             r_(P3V3_PCI_A_R.get()), g_(P3V3_PCI_A_G.get()),
             r_(P1V5_LOGIC_R.get()), g_(P1V5_LOGIC_G.get()),
             r_(BIT_R.get()), g_(BIT_BRIDGE_G.get()), g_(BIT_CPU0_G.get()), g_(BIT_CPU1_G.get()),
             r_(UNC3_R.get()), g_(UNC3_G.get()));
    println!("P3V3_B {} {} | P1V2_LOGIC {} {} | MEM LD {} {}       {} {} UNC4",
             r_(P3V3_PCI_B_R.get()), g_(P3V3_PCI_B_G.get()),
             r_(P1V2_LOGIC_R.get()), g_(P1V2_LOGIC_G.get()),
             r_(MEM_R.get()), g_(MEM_G.get()),
             r_(UNC4_R.get()), g_(UNC4_G.get()));
    println!("N12V   {} {} | PV75_TERM  {} {} | RUN    {} {}    {} {} {} UNC5",
             r_(N12V_PCI_R.get()), g_(N12V_PCI_G.get()),
             r_(PV75_TERM_R.get()), g_(PV75_TERM_G.get()),
             r_(RUN_R.get()), g_(RUN_G.get()), g_(UPDOG_G.get()),
             r_(UNC5_R.get()), g_(UNC5_G.get()));
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
    let temp_logic = try!(devices::SENSOR_LOGIC.read());
    let temp_ambient = try!(devices::SENSOR_AMBIENT.read());

    println!("Logic:   {}.{} degC", temp_logic/10, temp_logic%10);
    println!("Ambient: {}.{} degC", temp_ambient/10, temp_ambient%10);

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
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));

    let is_present = try!(devices::twi::TWI0.probe(addr));

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
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));
    let loc = try!(argv_parsed(args, 2, "LOCATION", u8::parseint));
    let n = try!(argv_parsed(args, 3, "N", u8::parseint));
    if n > 16 {
        return Err(ERR_ARG_RANGE);
    }

    let location_arr = [loc];
    let mut buffer = [0 as u8; 16];

    try!(devices::twi::TWI0
         .read(addr, &location_arr, &mut buffer[0..n as usize]));
    println!("{:?}", &buffer[0..n as usize]);
    Ok(())
}

fn cmd_i2c_write(args: &[&str]) -> StdResult
{
    if args.len() < 3 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let addr = try!(argv_parsed(args, 1, "ADDR", u8::parseint));
    let loc = try!(argv_parsed(args, 2, "LOCATION", u8::parseint));

    if args.len() > 19 {
        return Err(ERR_TOO_MANY_ARGS);
    }

    let mut buffer = [0 as u8; 16];
    let n = args.len() - 3;
    for i in 0..n {
        let arg = try!(argv_parsed(args, i + 3, "BYTES", u8::parseint));
        buffer[i] = arg;
    }

    let location_arr = [loc];

    try!(devices::twi::TWI0
         .write(addr, &location_arr, &buffer[0..n as usize]));
    Ok(())
}

fn cmd_gpio_read(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let gpio_name = args[1];

    match devices::pins::PIN_TABLE.iter().find(|&pin| {*(pin.name()) == *gpio_name}) {
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
    let gpio_val = try!(argv_parsed(args, 2, "VALUE", i8::parseint));

    match devices::pins::PIN_TABLE.iter().find(|&pin| {*(pin.name()) == *gpio_name}) {
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

    let load = try!(argv_parsed(args, 1, "PF", u32::parseint));

    devices::CLOCK_SYNTH.loadcap(load)
}

fn cmd_extclk(_args: &[&str]) -> StdResult
{
    unsafe{bindgen_mcu::mcu_use_external_clock(true);}
    Ok(())
}

fn cmd_intclk(_args: &[&str]) -> StdResult
{
    unsafe{bindgen_mcu::mcu_use_external_clock(false);}
    Ok(())
}

fn cmd_pwr_stat(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }
    let supply_name = args[1];
    let _lock = devices::supplies::POWER_MUTEX.lock();
    match devices::supplies::SUPPLY_TABLE.iter().find(|&supply| {*(supply.name()) == *supply_name}) {
        Some(supply) => println!("supply {} status: {:?}", supply_name, try!(supply.status())),
        None => println!("supply {} not found", supply_name),
    }
    Ok(())
}

static mut BLOCKDEV: Option<drivers::ext4::SdBlockDev> = None;

fn cmd_mount(_args: &[&str]) -> StdResult
{
    if !CARD.get() {
        return Err(ERR_NO_CARD);
    }

    CARDEN.set(true);
    os::delay(1);
    try!(devices::SD.lock().check());

    let mut gpt = drivers::gpt::Gpt::new(&devices::SD);
    let mut entry = drivers::gpt::GptEntry::new();

    try!(gpt.read_header());
    try!(gpt.read_boot(&mut entry));

    if !entry.valid() {
        return Err(ERR_NO_BOOT_PART);
    }

    unsafe {
        BLOCKDEV = Some(drivers::ext4::makedev(&devices::SD, &entry));
        try!(drivers::ext4::register_device(BLOCKDEV.as_mut().unwrap(), "root"));
    }

    try!(drivers::ext4::mount("root", "/", false));
    Ok(())
}

fn cmd_umount(_args: &[&str]) -> StdResult
{
    try!(drivers::ext4::umount("/"));
    try!(drivers::ext4::unregister_device("root"));

    if !CARD.get() {
        return Err(ERR_NO_CARD)
    }
    CARDEN.set(false);
    Ok(())
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
    println!("Protected: {}",
             if sd.writeprotected() { "yes" } else { "no" });

    Ok(())
}

fn cmd_readblock(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let iblock = try!(argv_parsed(args, 1, "BLOCK", u32::parseint)) as usize;

    let mut buf = [0u8; 512];
    try!(devices::SD.lock().read_block(iblock, &mut buf));

    hexprint(&buf);
    Ok(())
}

fn cmd_writeblock(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let iblock = try!(argv_parsed(args, 1, "BLOCK", u32::parseint)) as usize;

    let mut buf = [0u8; 512];
    for i in 2..args.len() {
        let data = try!(argv_parsed(args, i, "DATA", u8::parseint));
        buf[i - 2] = data;
    }

    try!(devices::SD.lock().write_block(iblock, &buf));
    Ok(())
}

fn cmd_partinfo(_args: &[&str]) -> StdResult
{
    let mut gpt = drivers::gpt::Gpt::new(&devices::SD);
    let mut entry = drivers::gpt::GptEntry::new();

    try!(gpt.read_header());

    println!("Disk GUID: {}", gpt.guid());

    for i in 0..gpt.number_entries() {
        try!(gpt.read_entry(i, &mut entry));
        if !entry.valid() {
            continue;
        }

        println!("Entry {}:", i);
        println!("  Type GUID:   {}", entry.type_guid);
        println!("  Unique GUID: {}", entry.part_guid);
        println!("  Range:       {:08x}...{:08x}", entry.start_lba, entry.end_lba);
        println!("  Attributes:  {:08x}", entry.attributes);
        println!("  Name:        {}", entry.name());
    }

    Ok(())
}

fn cmd_ls(args: &[&str]) -> StdResult
{
    let path = if args.len() == 2 { args[1] } else { "/" };

    // Use a stringbuilder to append each item to the path, for stat()
    let mut sb = StringBuilder::new();
    try!(sb.append(path));
    let pab = path.as_bytes();
    if pab[pab.len() - 1] != '/' as u8 {
        try!(sb.append("/"));
    }
    let only_path = sb.len();

    let mut dir = try!(drivers::ext4::dir_open(path));
    for de in dir.iter() {
        let name = try!(de.name());
        try!(sb.append(name));
        let stat = try!(drivers::ext4::stat(sb.as_ref()));
        sb.truncate(only_path);

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

    let mut file = try!(drivers::ext4::fopen_expand(
            path, drivers::ext4::OpenFlags::Read));
    let mut buf = [0u8; 512];

    let bytes = try!(file.read(&mut buf));

    hexprint(&buf[0..bytes]);
    Ok(())
}

fn cmd_spi_dump(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[1];
    let mut file = drivers::ext4::fopen_expand(path, drivers::ext4::OpenFlags::Read)?;

    let mut buf1 = vec::from_elem(0u8, 4096);
    let mut buf2 = vec::from_elem(0u8, 4096);

    let mut wr1 = None;

    loop {
        let n_read1 = try!(file.read(&mut buf1));

        if let Some(wr) = wr1.take() {
            devices::SPI.end_write(wr);
        }

        if n_read1 == 0 {
            break;
        }

        let wr2 = devices::SPI.start_write(&buf1[0..n_read1])?;

        let n_read2 = try!(file.read(&mut buf2));
        devices::SPI.end_write(wr2);

        if n_read2 == 0 {
            break;
        }

        wr1 = Some(devices::SPI.start_write(&buf2[0..n_read2])?);
    }

    Ok(())
}

fn cmd_readlink(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[1];

    let link = try!(drivers::ext4::readlink(path));

    println!("{}", link);
    Ok(())
}

fn cmd_expand(args: &[&str]) -> StdResult
{
    if args.len() < 2 {
        return Err(ERR_EXPECTED_ARGS);
    }

    let path = args[1];

    let path = try!(drivers::ext4::expand(path));

    println!("{}", path);
    Ok(())
}

fn cmd_ftrans(_args: &[&str]) -> StdResult
{
    let mut ftrans = FTrans::new();
    ftrans.run();
    Ok(())
}
