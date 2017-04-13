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

extern crate lwext4;
extern crate ctypes;
use core::ptr;
use core::mem;
use core::str;
use core::slice;
use self::lwext4::ext4_blockdev_iface;
pub use self::lwext4::ext4_blockdev;
use hardware::sd::*;
use main::gpt;
use rustsys::ec_io;

const EIO: i32 = 5;

static mut BLOCKDEV_BUF: [u8; 512] = [0u8; 512];
static mut BLOCKDEV_IFACE: ext4_blockdev_iface = ext4_blockdev_iface {
    open:   Some(blockdev_open),
    bread:  Some(blockdev_bread),
    bwrite: Some(blockdev_bwrite),
    close:  Some(blockdev_close),
    lock:   Some(blockdev_lock),
    unlock: Some(blockdev_unlock),
    ph_bsize:   512,
    ph_bcnt:    0,
    ph_bbuf:    unsafe{&BLOCKDEV_BUF as *const u8 as *mut u8},
    ph_refctr:  0,
    bread_ctr:  0,
    bwrite_ctr: 0,
};

extern "C" {
    fn pvPortMalloc(sz: usize) -> *mut u8;
    fn vPortFree(pv: *mut u8);
}

#[allow(unused)]
extern "C"
fn blockdev_open(bdev: *mut ext4_blockdev) -> i32
{
    0
}

#[allow(unused)]
unsafe extern "C"
fn blockdev_bread(bdev: *mut ext4_blockdev, buf: *mut ctypes::c_void,
                  blk_id: u64, blk_cnt: u32) -> i32
{
    match SD.lock().read_blocks(blk_id as usize, blk_cnt as u16, buf as *mut u8) {
        Ok(()) => 0,
        Err(_) => EIO,
    }
}

#[allow(unused)]
unsafe extern "C"
fn blockdev_bwrite(bdev: *mut ext4_blockdev, buf: *const ctypes::c_void,
                   blk_id: u64, blk_cnt: u32) -> i32
{
    match SD.lock().write_blocks(blk_id as usize, blk_cnt as u16, buf as *const u8) {
        Ok(()) => 0,
        Err(_) => EIO,
    }
}

#[allow(unused)]
extern "C"
fn blockdev_close(bdev: *mut ext4_blockdev) -> i32
{
    0
}

#[allow(unused)]
extern "C"
fn blockdev_lock(bdev: *mut ext4_blockdev) -> i32
{
    EIO
}

#[allow(unused)]
extern "C"
fn blockdev_unlock(bdev: *mut ext4_blockdev) -> i32
{
    EIO
}

pub fn makedev(part: &gpt::GptEntry) -> ext4_blockdev
{
    ext4_blockdev {
        bdif: unsafe{&mut BLOCKDEV_IFACE},
        part_offset: (512 * part.start_lba) as u64,
        part_size: (512 * (part.end_lba - part.start_lba + 1)) as u64,
        bc: ptr::null_mut(),
        lg_bsize: 0,
        lg_bcnt: 0,
        cache_write_back: 0,
        fs: ptr::null_mut(),
        journal: ptr::null_mut(),
    }
}

pub fn ls(bd: &mut ext4_blockdev) -> i32
{
    unsafe{
    if lwext4::ext4_device_register(bd, "root\0".as_ptr() as *const i8) != 0 { return 1; }
    if lwext4::ext4_mount("root\0".as_ptr() as *const i8, "/\0".as_ptr() as *const i8, false) != 0 { return 2; }

    let mut dir: lwext4::ext4_dir = mem::zeroed();
    lwext4::ext4_dir_open(&mut dir, "/\0".as_ptr() as *const i8);
    let mut de: *const lwext4::ext4_direntry = lwext4::ext4_dir_entry_next(&mut dir);

    while de != ptr::null() {
        let slice = slice::from_raw_parts(&(*de).name[0], (*de).name_length as usize);
        let s = str::from_utf8_unchecked(slice);
        println!("{}", s);
        de = lwext4::ext4_dir_entry_next(&mut dir);
    }
    lwext4::ext4_dir_close(&mut dir);

    if lwext4::ext4_umount("/\0".as_ptr() as *const i8) != 0 { return 100; }
    if lwext4::ext4_device_unregister("root\0".as_ptr() as *const i8) != 0 { return 101; }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_malloc(sz: usize) -> *mut u8
{
    pvPortMalloc(sz)
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_calloc(sz: usize) -> *mut u8
{
    let p = pvPortMalloc(sz);

    for i in 0..sz {
        *(p.offset(i as isize)) = 0;
    }

    p
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_free(pv: *mut u8) {
    vPortFree(pv)
}
