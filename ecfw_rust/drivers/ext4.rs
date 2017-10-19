// c4puter embedded controller firmware
// Copyright (C) 2017 Chris Pavlina
//
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

extern crate lwext4;
extern crate ctypes;
use core::{convert, fmt, mem, ops, ptr, slice, str};
use core::marker::PhantomData;
use alloc::raw_vec::RawVec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::string::String;
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

unsafe impl<'a> Sync for SdBlockDev<'a> {}
unsafe impl<'a> Send for SdBlockDev<'a> {}

const EIO: i32 = 5;

/// Error struct containing a number of bytes accessed before error, as well as
/// an error code.
pub struct IoError {
    error: Error,
    bytes: usize,
}

impl ops::Deref for IoError {
    type Target = Error;

    fn deref(&self) -> &Error
    {
        &self.error
    }
}

impl convert::From<IoError> for Error {
    fn from(e: IoError) -> Error
    {
        e.error()
    }
}

impl IoError {
    pub const fn new(error: Error, bytes: usize) -> IoError
    {
        IoError {
            error: error,
            bytes: bytes,
        }
    }

    pub fn bytes(&self) -> usize
    {
        self.bytes
    }

    pub fn error(&self) -> Error
    {
        self.error
    }
}

///////////////////////////////////////////////////////////////////////////////
// Block device registration - lifetime notes
// lwext4 takes ownership of the pointer passed to it, but doesn't give a way
// to get the pointer back when unregistering. This makes for less than
// beautiful mapping to Rust lifetimes.
//
// This is resolved by only allowing one block device to be registered at a
// time. The single block device (BLOCKDEV) has a mutex guarding it, but
// be warned: this is only to guard the Option<> - if the Option is Some,
// then the block device is also owned by lwext4 until unregistered.
//
// There are two better ways to handle this, neither of which I'm going to do:
//
// 1. Instead of a single static block device, have a static map of device
//    names and structs. When unregistering a device, it can be found in the
//    map and deleted. Not doing because it's wasteful.
//
// 2. Modify lwext4 to relinquish ownership and return the pointer when
//    unregistering. Instead of using static device structs, use a Box, which
//    is converted to a pointer for lwext4, then reboxed and dropped on
//    unregister. This is cleanest but I can't be arsed to submit a PR.

static BLOCKDEV: Mutex<Option<SdBlockDev<'static>>> = Mutex::new(None);

/// Register a block device with a device name.
pub fn register_device(bd: SdBlockDev<'static>, dev_name: &str) -> StdResult
{
    let mut lock = BLOCKDEV.lock();

    if lock.is_some() {
        return Err(ERR_EEXIST);
    }

    *lock = Some(bd);

    let mut alloc = StrAlloc::new();
    let c_name = alloc.nulterm(dev_name)?.as_ptr() as *const _;

    debug!(DEBUG_FS, "register block device \"{}\"", dev_name);

    let devptr = lock.as_mut().unwrap().to_ptr();
    let rc = unsafe { lwext4::ext4_device_register(devptr, c_name) };

    to_stdresult(rc)
}

/// Unregister a block device.
pub fn unregister_device(dev_name: &str) -> StdResult
{
    let mut lock = BLOCKDEV.lock();

    if lock.is_none() {
        return Err(ERR_ENOENT);
    }

    let mut alloc = StrAlloc::new();
    let c_name = alloc.nulterm(dev_name)?.as_ptr() as *const _;

    debug!(DEBUG_FS, "unregister block device \"{}\"", dev_name);

    // Ignore the result. For some reason this ALWAYS returns ENOENT.
    unsafe { lwext4::ext4_device_unregister(c_name) };

    *lock = None;
    Ok(())
}

/// Unregister all block devices.
pub fn unregister_all() -> StdResult
{
    debug!(DEBUG_FS, "unregister all block devices");
    to_stdresult(unsafe { lwext4::ext4_device_unregister_all() })
}

/// Mount a filesystem. If journaled, recovers journal.
pub fn mount(dev_name: &str, mount_point: &str, read_only: bool) -> StdResult
{
    let journaled;
    let mut alloc = StrAlloc::new();
    let c_name = alloc.nulterm(dev_name)?.as_ptr() as *const _;
    let c_mp = alloc.nulterm(mount_point)?.as_ptr() as *const _;

    debug!(DEBUG_FS, "mount \"{}\" as \"{}\"", dev_name, mount_point);
    to_stdresult(unsafe { lwext4::ext4_mount(c_name, c_mp, read_only) })?;

    debug!(DEBUG_FS, "recover journal on \"{}\"", mount_point);
    match to_stdresult(unsafe { lwext4::ext4_recover(c_mp) }) {
        Ok(_) => {
            journaled = true;
        },
        Err(e) if e == ERR_ENOTSUP => {
            debug!(DEBUG_FS, "filesystem \"{}\" has no journal", mount_point);
            journaled = false;
        },
        Err(e) => {
            return Err(e);
        },
    }

    if journaled {
        debug!(DEBUG_FS, "start journal on \"{}\"", mount_point);
        to_stdresult(unsafe { lwext4::ext4_journal_start(c_mp) })?;
    }

    to_stdresult(unsafe { lwext4::ext4_cache_write_back(c_mp, true) })?;

    Ok(())
}

/// Unmount a filesystem.
pub fn umount(mount_point: &str) -> StdResult
{
    let mut alloc = StrAlloc::new();
    let c_mp = alloc.nulterm(mount_point)?.as_ptr() as *const _;

    debug!(DEBUG_FS, "flush cache on \"{}\"", mount_point);
    to_stdresult(unsafe { lwext4::ext4_cache_write_back(c_mp, false) })?;

    debug!(DEBUG_FS, "stop journal on \"{}\", if any", mount_point);
    to_stdresult(unsafe { lwext4::ext4_journal_stop(c_mp) })?;

    debug!(DEBUG_FS, "umount \"{}\"", mount_point);
    to_stdresult(unsafe { lwext4::ext4_umount(c_mp) })?;

    Ok(())
}

/// Flush a filesystem's cache.
pub fn sync(mount_point: &str) -> StdResult
{
    let mut alloc = StrAlloc::new();
    let c_mp = alloc.nulterm(mount_point)?.as_ptr() as *const _;

    debug!(DEBUG_FS, "flush cache on \"{}\"", mount_point);
    to_stdresult(unsafe { lwext4::ext4_cache_flush(c_mp) })
}

/// Open a directory.
pub fn dir_open(path: &str) -> Result<Dir, Error>
{
    let mut alloc = StrAlloc::new();
    let c_path = alloc.nulterm(path)?.as_ptr() as *const _;

    let mut dir: Dir = unsafe { mem::zeroed() };
    to_stdresult(unsafe { lwext4::ext4_dir_open(&mut dir.0, c_path) })?;

    Ok(dir)
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum OpenFlags {
    Read = lwext4::O_RDONLY,
    Write = lwext4::O_WRONLY | lwext4::O_CREAT | lwext4::O_TRUNC,
    Append = lwext4::O_WRONLY | lwext4::O_CREAT | lwext4::O_APPEND,
    ReadWrite = lwext4::O_RDWR,
    ReadTruncate = lwext4::O_RDWR | lwext4::O_CREAT | lwext4::O_TRUNC,
    ReadAppend = lwext4::O_RDWR | lwext4::O_CREAT | lwext4::O_APPEND,
}

/// Open a file.
pub fn fopen(path: &str, flags: OpenFlags) -> Result<File, Error>
{
    let mut alloc = StrAlloc::new();
    let c_path = alloc.nulterm(path)?.as_ptr();
    fopen_cstr(c_path, flags)
}

/// Open a file from a C string path. Used internally to reduce the allocations
/// caused by additional null-termination when the string is already terminated
/// or can be terminated cheaply.
fn fopen_cstr(path: *const u8, flags: OpenFlags) -> Result<File, Error>
{
    let c_path = path as *const _;
    let mut file: File = unsafe { mem::zeroed() };
    to_stdresult(unsafe {
        lwext4::ext4_fopen2(&mut file.0, c_path, flags as _)
    })?;

    Ok(file)
}

/// Open a file, expanding symlinks in the path first.
pub fn fopen_expand(path: &str, flags: OpenFlags) -> Result<File, Error>
{
    let mut expanded = expand(path)?;

    // Append \0 to get a C string
    expanded.push('\0');
    fopen_cstr(expanded.as_ptr(), flags)
}

/// Stat a file.
pub fn stat(path: &str) -> Result<Stat, Error>
{
    let mut alloc = StrAlloc::new();
    let c_path = alloc.nulterm(path)?.as_ptr();
    stat_cstr(c_path)
}

/// Stat a file from a C string path. Used internally to reduce the allocations
/// caused by additional null-termination when the string is already terminated
/// or can be terminated cheaply.
fn stat_cstr(path: *const u8) -> Result<Stat, Error>
{
    let c_path = path as *const _;
    let mut inode: Stat = unsafe { mem::zeroed() };
    let mut ret_ino = 0u32;

    to_stdresult(unsafe {
        lwext4::ext4_raw_inode_fill(c_path, &mut ret_ino, &mut inode.0)
    })?;

    Ok(inode as Stat)
}

#[repr(C)]
pub struct Stat(lwext4::ext4_inode);

/// Read a link.
pub fn readlink(path: &str) -> Result<String, Error>
{
    let mut alloc = StrAlloc::new();
    let c_path = alloc.nulterm(path)?.as_ptr() as *const _;

    let mut buf = vec::from_elem(0u8, 1024);

    let rc = unsafe {
        lwext4::ext4_readlink(
            c_path,
            buf.as_mut_slice().as_mut_ptr() as *mut _,
            buf.len(),
            ptr::null_mut(),
        )
    };

    to_stdresult(rc)?;

    for i in 0 .. buf.len() {
        if buf[i] == 0 {
            buf.truncate(i);
            break;
        }
    }

    match String::from_utf8(buf) {
        Ok(s) => Ok(s),
        Err(_) => Err(ERR_UTF8),
    }
}

/// Unlink a path
pub fn unlink(path: &str) -> StdResult
{
    let mut alloc = StrAlloc::new();
    let c_path = alloc.nulterm(path)?.as_ptr() as *const _;

    let rc = unsafe { lwext4::ext4_fremove(c_path) };
    to_stdresult(rc)
}

/// Expand a path, following all symlinks.
pub fn expand(path: &str) -> Result<String, Error>
{
    let mut s = String::with_capacity(1024);

    for i in path.split('/') {
        if i.len() == 0 {
            continue;
        };

        // In order to stat this path element, we append it to the string
        // builder, stat that path, and then truncate it back off.
        let len = s.len();
        s.push('/');
        s.push_str(i);
        s.push('\0');

        let stat = stat_cstr(s.as_ptr())?;

        // Truncate the \0 that was added just to get a C string
        let without_nulterm = s.len() - 1;
        s.truncate(without_nulterm);

        if stat.inode_type() == InodeType::Symlink {
            let link = readlink(&s)?;

            if link.as_bytes()[0] == '/' as u8 {
                s.truncate(0);
            } else {
                s.truncate(len);
                s.push('/');
            }
            s.push_str(&link);
        }
    }

    if s.len() == 0 && path.len() > 0 {
        // Special case, we were given "/" or similar ("///", etc). These will
        // produce an empty stringbuilder but should expand to "/"
        s.push('/');
    }

    Ok(s)
}

#[repr(C)]
pub struct Dir(lwext4::ext4_dir);

impl Dir {
    pub fn iter(&mut self) -> DirIter
    {
        unsafe { lwext4::ext4_dir_entry_rewind(&mut self.0) };
        DirIter { dir: self }
    }
}

impl ops::Drop for Dir {
    fn drop(&mut self)
    {
        to_stdresult(unsafe { lwext4::ext4_dir_close(&mut self.0) })
            .unwrap();
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
        let slice = unsafe {
            slice::from_raw_parts(
                &(*self.de).name[0],
                (*self.de).name_length as usize,
            )
        };
        let s = str::from_utf8(slice).or(Err(ERR_UTF8))?;
        Ok(s)
    }
}

pub struct DirIter<'a> {
    dir: &'a mut Dir,
}

impl<'a> Iterator for DirIter<'a> {
    type Item = DirEntry<'a>;

    fn next(&mut self) -> Option<DirEntry<'a>>
    {
        let de = unsafe { lwext4::ext4_dir_entry_next(&mut self.dir.0) };

        if de == ptr::null() {
            None
        } else {
            Some(DirEntry {
                de: de,
                _p: PhantomData,
            })
        }
    }
}

#[repr(C)]
pub struct File(lwext4::ext4_file);

pub enum Origin {
    Set,
    Current,
    End,
}

impl File {
    /// Truncate file to the specified length.
    pub fn truncate(&mut self, size: usize) -> StdResult
    {
        to_stdresult(
            unsafe { lwext4::ext4_ftruncate(&mut self.0, size as u64) },
        )
    }

    /// Read data from file. Will attempt to fill `buf`; returns the number
    /// of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError>
    {
        let mut rcnt = 0usize;
        let buflen = buf.len();

        match to_stdresult(unsafe {
            lwext4::ext4_fread(
                &mut self.0,
                buf.as_mut_ptr() as *mut ctypes::c_void,
                buflen,
                &mut rcnt,
            )
        }) {

            Ok(..) => Ok(rcnt),
            Err(e) => Err(IoError::new(e, rcnt)),
        }
    }

    /// Write data to file. Will attempt to write all of `buf`; returns the
    /// number of bytes written.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, IoError>
    {
        let mut rcnt = 0usize;
        let buflen = buf.len();

        match to_stdresult(unsafe {
            lwext4::ext4_fwrite(
                &mut self.0,
                buf.as_ptr() as *mut ctypes::c_void,
                buflen,
                &mut rcnt,
            )
        }) {

            Ok(..) => Ok(rcnt),
            Err(e) => Err(IoError::new(e, rcnt)),
        }
    }

    /// Seek to a position.
    pub fn seek(&mut self, offset: usize, origin: Origin) -> StdResult
    {
        let c_origin = match origin {
            Origin::Set => lwext4::SEEK_SET,
            Origin::Current => lwext4::SEEK_CUR,
            Origin::End => lwext4::SEEK_END,
        };

        to_stdresult(unsafe {
            lwext4::ext4_fseek(&mut self.0, offset as u64, c_origin)
        })
    }

    /// Get file position
    pub fn tell(&mut self) -> usize
    {
        (unsafe { lwext4::ext4_ftell(&mut self.0) }) as usize
    }

    /// Get file size
    pub fn size(&mut self) -> usize
    {
        (unsafe { lwext4::ext4_fsize(&mut self.0) }) as usize
    }
}

impl ops::Drop for File {
    fn drop(&mut self)
    {
        to_stdresult(unsafe { lwext4::ext4_fclose(&mut self.0) })
            .unwrap();
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub enum InodeType {
    Other = 0,
    Fifo = 0x1000,
    Chardev = 0x2000,
    Dir = 0x4000,
    Blockdev = 0x6000,
    File = 0x8000,
    Symlink = 0xA000,
    Socket = 0xC000,
}

impl Stat {
    /// Get mode. Warning, HURD has 32-bit mode and we ignore upper bits.
    pub fn mode(&self) -> u16
    {
        u16::from_le(self.0.mode)
    }

    pub fn uid(&self) -> u16
    {
        u16::from_le(self.0.uid)
    }

    pub fn gid(&self) -> u16
    {
        u16::from_le(self.0.gid)
    }

    pub fn atime(&self) -> u32
    {
        u32::from_le(self.0.access_time)
    }

    pub fn ctime(&self) -> u32
    {
        u32::from_le(self.0.change_inode_time)
    }

    pub fn mtime(&self) -> u32
    {
        u32::from_le(self.0.modification_time)
    }

    pub fn linkcount(&self) -> u16
    {
        u16::from_le(self.0.links_count)
    }

    pub fn flags(&self) -> u32
    {
        u32::from_le(self.0.flags)
    }

    pub fn size(&self) -> u32
    {
        u32::from_le(self.0.size_lo)
    }

    pub fn inode_type(&self) -> InodeType
    {
        let mode = self.mode();
        let typebits = mode as u32 & lwext4::EXT4_INODE_MODE_TYPE_MASK;

        match typebits {
            lwext4::EXT4_INODE_MODE_FIFO => InodeType::Fifo,
            lwext4::EXT4_INODE_MODE_CHARDEV => InodeType::Chardev,
            lwext4::EXT4_INODE_MODE_DIRECTORY => InodeType::Dir,
            lwext4::EXT4_INODE_MODE_BLOCKDEV => InodeType::Blockdev,
            lwext4::EXT4_INODE_MODE_FILE => InodeType::File,
            lwext4::EXT4_INODE_MODE_SOFTLINK => InodeType::Symlink,
            lwext4::EXT4_INODE_MODE_SOCKET => InodeType::Socket,
            _ => InodeType::Other,
        }
    }
}

/// When printed, Stat emits an ls-style mode readout.
impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let t = match self.inode_type() {
            InodeType::Fifo => "p",
            InodeType::Chardev => "c",
            InodeType::Dir => "d",
            InodeType::Blockdev => "b",
            InodeType::File => "-",
            InodeType::Symlink => "l",
            InodeType::Socket => "s",
            InodeType::Other => "X",
        };

        let mode = self.mode();

        fn rwx(n: u16) -> &'static str
        {
            match n & 0b111 {
                0b000 => "---",
                0b001 => "--x",
                0b010 => "-w-",
                0b011 => "-wx",
                0b100 => "r--",
                0b101 => "r-x",
                0b110 => "rw-",
                0b111 => "rwx",
                _ => "---", // stupid compiler
            }
        }

        write!(f, "{}", t)?;
        write!(f, "{}", rwx(mode >> 6))?;
        write!(f, "{}", rwx(mode >> 3))?;
        write!(f, "{}", rwx(mode >> 0))?;
        Ok(())
    }
}

static mut BLOCKDEV_BUF: [u8; 512] = [0u8; 512];

static mut BLOCKDEV_IFACE: ext4_blockdev_iface = ext4_blockdev_iface {
    open: Some(blockdev_open),
    bread: Some(blockdev_bread),
    bwrite: Some(blockdev_bwrite),
    close: Some(blockdev_close),
    lock: Some(blockdev_lock),
    unlock: Some(blockdev_unlock),
    ph_bsize: 512,
    ph_bcnt: 0,
    ph_bbuf: unsafe { &BLOCKDEV_BUF } as *const _ as *mut u8,
    ph_refctr: 0,
    bread_ctr: 0,
    bwrite_ctr: 0,
};

impl<'a> SdBlockDev<'a> {
    fn from_ptr(p: *mut ext4_blockdev) -> &'a mut SdBlockDev<'a>
    {
        unsafe { mem::transmute(p) }
    }

    fn to_ptr(&mut self) -> *mut ext4_blockdev
    {
        &mut self.lwext4_bd
    }
}

extern "C" fn blockdev_open(_bdev: *mut ext4_blockdev) -> i32
{
    0
}

unsafe extern "C" fn blockdev_bread(
    bdev: *mut ext4_blockdev,
    buf: *mut ctypes::c_void,
    blk_id: u64,
    blk_cnt: u32,
) -> i32
{
    let bd = SdBlockDev::from_ptr(bdev);
    let mut sd = bd.sd.lock();
    match sd.read_blocks(
        blk_id as usize,
        blk_cnt as u16,
        buf as *mut u8,
    ) {
        Ok(()) => 0,
        Err(_) => EIO,
    }
}

unsafe extern "C" fn blockdev_bwrite(
    bdev: *mut ext4_blockdev,
    buf: *const ctypes::c_void,
    blk_id: u64,
    blk_cnt: u32,
) -> i32
{
    let bd = SdBlockDev::from_ptr(bdev);
    let mut sd = bd.sd.lock();
    match sd.write_blocks(
        blk_id as usize,
        blk_cnt as u16,
        buf as *const u8,
    ) {
        Ok(()) => 0,
        Err(_) => EIO,
    }
}

extern "C" fn blockdev_close(_bdev: *mut ext4_blockdev) -> i32
{
    0
}

extern "C" fn blockdev_lock(_bdev: *mut ext4_blockdev) -> i32
{
    EIO
}

extern "C" fn blockdev_unlock(_bdev: *mut ext4_blockdev) -> i32
{
    EIO
}

pub fn makedev<'a>(sd: &'a Mutex<Sd>, part: &gpt::GptEntry) -> SdBlockDev<'a>
{
    SdBlockDev {
        lwext4_bd: ext4_blockdev {
            bdif: unsafe { &mut BLOCKDEV_IFACE },
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
    let rv = RawVec::with_capacity(sz);
    &mut ((*Box::into_raw(rv.into_box()))[0])
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_calloc(n: usize, sz: usize) -> *mut u8
{
    let p = ext4_user_malloc(sz * n);

    for i in 0 .. (sz * n) {
        *(p.offset(i as isize)) = 0;
    }

    p
}

#[no_mangle]
pub unsafe extern "C" fn ext4_user_free(pv: *mut u8)
{
    let _b = Box::from_raw(pv);
    // _b is freed on drop
}

/// Map errno returns from lwext4 to StdResult.
fn to_stdresult(ec: i32) -> StdResult
{
    match ec {
        0 => Ok(()),
        1 => Err(ERR_EPERM),
        2 => Err(ERR_ENOENT),
        5 => Err(ERR_EIO),
        6 => Err(ERR_ENXIO),
        7 => Err(ERR_E2BIG),
        12 => Err(ERR_ENOMEM),
        13 => Err(ERR_EACCES),
        14 => Err(ERR_EFAULT),
        17 => Err(ERR_EEXIST),
        19 => Err(ERR_ENODEV),
        20 => Err(ERR_ENOTDIR),
        21 => Err(ERR_EISDIR),
        22 => Err(ERR_EINVAL),
        27 => Err(ERR_EFBIG),
        28 => Err(ERR_ENOSPC),
        30 => Err(ERR_EROFS),
        31 => Err(ERR_EMLINK),
        34 => Err(ERR_ERANGE),
        39 => Err(ERR_ENOTEMPTY),
        61 => Err(ERR_ENODATA),
        95 => Err(ERR_ENOTSUP),
        _ => Err(ERR_UNKNOWN),
    }
}
