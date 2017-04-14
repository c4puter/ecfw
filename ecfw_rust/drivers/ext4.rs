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
use core::{ptr, mem, str, slice, ops, convert};
use core::marker::PhantomData;
use alloc::raw_vec::RawVec;
use alloc::boxed::Box;
use self::lwext4::ext4_blockdev_iface;
pub use self::lwext4::ext4_blockdev;

use drivers::sd::*;
use drivers::gpt;
use os::{Mutex, StrAlloc};
use messages::*;

#[repr(C)]
pub struct SdBlockDev<'a> {
    lwext4_bd: ext4_blockdev,
    sd: &'a Mutex<Sd>,
}

const EIO: i32 = 5;

/// Error struct containing a number of bytes accessed before error, as well as
/// an error code.
pub struct IoError {
    error: Error,
    bytes: usize,
}

impl ops::Deref for IoError {
    type Target = Error;

    fn deref(&self) -> &Error {
        &self.error
    }
}

impl convert::From<IoError> for Error {
    fn from(e: IoError) -> Error {
        e.error()
    }
}

impl IoError {
    pub const fn new(error: Error, bytes: usize) -> IoError {
        IoError { error: error, bytes: bytes }
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn error(&self) -> Error {
        self.error
    }
}

/// Register a block device with a device name.
pub fn register_device(bd: &mut SdBlockDev, dev_name: &str) -> StdResult
{
    let mut alloc = StrAlloc::new();
    let c_name = try!(alloc.nulterm(dev_name)).as_ptr() as *const i8;

    debug!(DEBUG_FS, "register block device \"{}\"", dev_name);
    to_stdresult(unsafe{lwext4::ext4_device_register(bd.to_ptr(), c_name)})
}

/// Unregister a block device.
pub fn unregister_device(dev_name: &str) -> StdResult
{
    let mut alloc = StrAlloc::new();
    let c_name = try!(alloc.nulterm(dev_name)).as_ptr() as *const i8;

    debug!(DEBUG_FS, "unregister block device \"{}\"", dev_name);

    // Ignore the result. For some reason this ALWAYS returns ENOENT.
    unsafe{lwext4::ext4_device_unregister(c_name)};
    Ok(())
}

/// Unregister all block devices.
pub fn unregister_all() -> StdResult
{
    debug!(DEBUG_FS, "unregister all block devices");
    to_stdresult(unsafe{lwext4::ext4_device_unregister_all()})
}

/// Mount a filesystem. If journaled, recovers journal.
pub fn mount(dev_name: &str, mount_point: &str, read_only: bool) -> StdResult
{
    let journaled;
    let mut alloc = StrAlloc::new();
    let c_name = try!(alloc.nulterm(dev_name)).as_ptr() as *const i8;
    let c_mp = try!(alloc.nulterm(mount_point)).as_ptr() as *const i8;

    debug!(DEBUG_FS, "mount \"{}\" as \"{}\"", dev_name, mount_point);
    try!(to_stdresult(unsafe{lwext4::ext4_mount(c_name, c_mp, read_only)}));

    debug!(DEBUG_FS, "recover journal on \"{}\"", mount_point);
    match to_stdresult(unsafe{lwext4::ext4_recover(c_mp)}) {
        Ok(_) => { journaled = true; },
        Err(e) if e == ERR_ENOTSUP => {
            debug!(DEBUG_FS, "filesystem \"{}\" has no journal", mount_point);
            journaled = false;
        },
        Err(e) => { return Err(e); },
    }

    if journaled {
        debug!(DEBUG_FS, "start journal on \"{}\"", mount_point);
        try!(to_stdresult(unsafe{lwext4::ext4_journal_start(c_mp)}));
    }

    try!(to_stdresult(unsafe{lwext4::ext4_cache_write_back(c_mp, true)}));

    Ok(())
}

/// Unmount a filesystem.
pub fn umount(mount_point: &str) -> StdResult
{
    let mut alloc = StrAlloc::new();
    let c_mp = try!(alloc.nulterm(mount_point)).as_ptr() as *const i8;

    debug!(DEBUG_FS, "flush cache on \"{}\"", mount_point);
    try!(to_stdresult(unsafe{lwext4::ext4_cache_write_back(c_mp, false)}));

    debug!(DEBUG_FS, "stop journal on \"{}\", if any", mount_point);
    try!(to_stdresult(unsafe{lwext4::ext4_journal_stop(c_mp)}));

    debug!(DEBUG_FS, "umount \"{}\"", mount_point);
    try!(to_stdresult(unsafe{lwext4::ext4_umount(c_mp)}));

    Ok(())
}

/// Open a directory.
pub fn dir_open(path: &str) -> Result<Dir,Error>
{
    let mut alloc = StrAlloc::new();
    let c_path = try!(alloc.nulterm(path)).as_ptr() as *const i8;

    let mut dir: Dir = unsafe{mem::zeroed()};
    try!(to_stdresult(unsafe{lwext4::ext4_dir_open(&mut dir.0, c_path)}));

    Ok(dir)
}

/// Open a file.
///
/// Flags: r, rb, w, wb, a, ab, r+, rb+, r+b, w+, wb+, w+b, a+, ab+, a+b
pub fn fopen(path: &str, flags: &str) -> Result<File,Error>
{
    let mut alloc = StrAlloc::new();
    let c_path = try!(alloc.nulterm(path)).as_ptr() as *const i8;
    let c_flags = try!(alloc.nulterm(flags)).as_ptr() as *const i8;

    let mut file: File = unsafe{mem::zeroed()};
    try!(to_stdresult(unsafe{lwext4::ext4_fopen(&mut file.0, c_path, c_flags)}));

    Ok(file)
}

#[repr(C)]
pub struct Dir(lwext4::ext4_dir);

impl Dir {
    pub fn iter(&mut self) -> DirIter
    {
        unsafe{lwext4::ext4_dir_entry_rewind(&mut self.0)};
        DirIter {
            dir: self
        }
    }
}

impl ops::Drop for Dir {
    fn drop(&mut self)
    {
        to_stdresult(unsafe{lwext4::ext4_dir_close(&mut self.0)}).unwrap();
    }
}

#[repr(C)]
pub struct DirEntry<'a> {
    de: *const lwext4::ext4_direntry,
    _p: PhantomData<&'a DirEntry<'a>>,
}

impl<'a> DirEntry<'a> {
    pub fn name(&'a self) -> Result<&'a str, Error>
    {
        let slice = unsafe{slice::from_raw_parts(&(*self.de).name[0],
                                          (*self.de).name_length as usize)};
        let s = try!(str::from_utf8(slice).or(Err(ERR_UTF8)));
        Ok(s)
    }
}

pub struct DirIter<'a> {
    dir: &'a mut Dir
}

impl<'a> Iterator for DirIter<'a> {
    type Item = DirEntry<'a>;

    fn next(&mut self) -> Option<DirEntry<'a>>
    {
        let de = unsafe{lwext4::ext4_dir_entry_next(&mut self.dir.0)};

        if de == ptr::null() { None }
        else {
            Some(DirEntry { de: de, _p: PhantomData } ) }
    }
}

#[repr(C)]
pub struct File(lwext4::ext4_file);

pub enum Origin {
    Set, Current, End
}

impl File {
    /// Truncate file to the specified length.
    pub fn truncate(&mut self, size: usize) -> StdResult
    {
        to_stdresult(unsafe{lwext4::ext4_ftruncate(&mut self.0, size as u64)})
    }

    /// Read data from file. Will attempt to fill `buf`; returns the number of
    /// bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize,IoError>
    {
        let mut rcnt = 0usize;
        let buflen = buf.len();

        match to_stdresult(unsafe{
            lwext4::ext4_fread(&mut self.0,
                               buf.as_mut_ptr() as *mut ctypes::c_void,
                               buflen, &mut rcnt)}) {

            Ok(..) => { Ok(rcnt) },
            Err(e) => { Err(IoError::new(e, rcnt)) },
        }
    }

    /// Write data to file. Will attempt to write all of `buf`; returns the
    /// number of bytes written.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize,IoError>
    {
        let mut rcnt = 0usize;
        let buflen = buf.len();

        match to_stdresult(unsafe{
            lwext4::ext4_fwrite(&mut self.0,
                                buf.as_ptr() as *mut ctypes::c_void,
                                buflen, &mut rcnt)}) {

            Ok(..) => { Ok(rcnt) },
            Err(e) => { Err(IoError::new(e, rcnt)) },
        }
    }

    /// Seek to a position.
    pub fn seek(&mut self, offset: usize, origin: Origin) -> StdResult
    {
        let c_origin = match origin {
            Origin::Set => lwext4::SEEK_SET,
            Origin::Current => lwext4::SEEK_CUR,
            Origin::End => lwext4::SEEK_END };

        to_stdresult(unsafe{
            lwext4::ext4_fseek(&mut self.0,
                               offset as u64,
                               c_origin)})
    }

    /// Get file position
    pub fn tell(&mut self) -> usize
    {
        (unsafe{lwext4::ext4_ftell(&mut self.0)}) as usize
    }

    /// Get file size
    pub fn size(&mut self) -> usize
    {
        (unsafe{lwext4::ext4_fsize(&mut self.0)}) as usize
    }
}

impl ops::Drop for File {
    fn drop(&mut self)
    {
        to_stdresult(unsafe{lwext4::ext4_fclose(&mut self.0)}).unwrap();
    }
}

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
    ph_bbuf:    unsafe{&BLOCKDEV_BUF} as *const _ as *mut u8,
    ph_refctr:  0,
    bread_ctr:  0,
    bwrite_ctr: 0,
};

impl<'a> SdBlockDev<'a> {
    fn from_ptr(p: *mut ext4_blockdev) -> &'a mut SdBlockDev<'a>
    {
        unsafe{ mem::transmute::<_, &mut SdBlockDev<'a>>(p) }
    }

    fn to_ptr(&mut self) -> *mut ext4_blockdev
    {
        &mut self.lwext4_bd
    }
}

extern "C"
fn blockdev_open(_bdev: *mut ext4_blockdev) -> i32
{
    0
}

unsafe extern "C"
fn blockdev_bread(bdev: *mut ext4_blockdev, buf: *mut ctypes::c_void,
                  blk_id: u64, blk_cnt: u32) -> i32
{
    let bd = SdBlockDev::from_ptr(bdev);
    let mut sd = bd.sd.lock();
    match sd.read_blocks(blk_id as usize, blk_cnt as u16, buf as *mut u8) {
        Ok(()) => 0,
        Err(_) => EIO,
    }
}

unsafe extern "C"
fn blockdev_bwrite(bdev: *mut ext4_blockdev, buf: *const ctypes::c_void,
                   blk_id: u64, blk_cnt: u32) -> i32
{
    let bd = SdBlockDev::from_ptr(bdev);
    let mut sd = bd.sd.lock();
    match sd.write_blocks(blk_id as usize, blk_cnt as u16, buf as *const u8) {
        Ok(()) => 0,
        Err(_) => EIO,
    }
}

extern "C"
fn blockdev_close(_bdev: *mut ext4_blockdev) -> i32
{
    0
}

extern "C"
fn blockdev_lock(_bdev: *mut ext4_blockdev) -> i32
{
    EIO
}

extern "C"
fn blockdev_unlock(_bdev: *mut ext4_blockdev) -> i32
{
    EIO
}

pub fn makedev<'a>(sd: &'a Mutex<Sd>, part: &gpt::GptEntry) -> SdBlockDev<'a>
{
    SdBlockDev {
        lwext4_bd: ext4_blockdev {
            bdif: unsafe{&mut BLOCKDEV_IFACE},
            part_offset: (512 * part.start_lba) as u64,
            part_size: (512 * (part.end_lba - part.start_lba + 1)) as u64,
            bc: ptr::null_mut(),
            lg_bsize: 0,
            lg_bcnt: 0,
            cache_write_back: 0,
            fs: ptr::null_mut(),
            journal: ptr::null_mut(),
        },
        sd: sd,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_malloc(sz: usize) -> *mut u8
{
    let rv = RawVec::<u8>::with_capacity(sz);
    &mut ((*Box::into_raw(rv.into_box()))[0])
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_calloc(n: usize, sz: usize) -> *mut u8
{
    let p = ext4_user_malloc(sz * n);

    for i in 0..(sz * n) {
        *(p.offset(i as isize)) = 0;
    }

    p
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_free(pv: *mut u8) {
    let _b = Box::from_raw(pv);
    // _b is freed on drop
}

/// Map errno returns from lwext4 to StdResult.
fn to_stdresult(ec: i32) -> StdResult
{
    match ec {
        0   =>  Ok(()),
        1   =>  Err(ERR_EPERM),
        2   =>  Err(ERR_ENOENT),
        5   =>  Err(ERR_EIO),
        6   =>  Err(ERR_ENXIO),
        7   =>  Err(ERR_E2BIG),
        12  =>  Err(ERR_ENOMEM),
        13  =>  Err(ERR_EACCES),
        14  =>  Err(ERR_EFAULT),
        17  =>  Err(ERR_EEXIST),
        19  =>  Err(ERR_ENODEV),
        20  =>  Err(ERR_ENOTDIR),
        21  =>  Err(ERR_EISDIR),
        22  =>  Err(ERR_EINVAL),
        27  =>  Err(ERR_EFBIG),
        28  =>  Err(ERR_ENOSPC),
        30  =>  Err(ERR_EROFS),
        31  =>  Err(ERR_EMLINK),
        34  =>  Err(ERR_ERANGE),
        39  =>  Err(ERR_ENOTEMPTY),
        61  =>  Err(ERR_ENODATA),
        95  =>  Err(ERR_ENOTSUP),
        _   =>  Err(ERR_UNKNOWN),
    }
}
