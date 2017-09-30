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
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
 * OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
 * OR OTHER DEALINGS IN THE SOFTWARE.
 */

extern crate lwext4_crc32;
extern crate ctypes;
use data::{StringBuilder,base64,ParseInt};
use drivers::ext4;
use rustsys::ec_io;
use core::iter::Iterator;
use core::str;
use messages::*;

pub struct FTrans {
    file: Option<ext4::File>,
}

impl FTrans {
    pub fn new() -> FTrans {
        FTrans{ file: None }
    }

    /// Open a file transfer session. Quits when either ^C or ^D is received.
    pub fn run(&mut self) {
        ec_io::flush_output();
        let mut sb = StringBuilder::new();
        let mut overflowed = false;
        let mut invalid = false;

        loop {
            let c = ec_io::getc_async();

            match c {
                3 | 4 => { /* ^C or ^D */ break; },
                0x20 ... 0x7e => {
                    overflowed = sb.append_char(c as char).is_err();
                },
                b'\r' => {
                    if overflowed {
                        self.handle_overflow();
                    } else if invalid {
                        self.handle_invalid();
                    } else {
                        if let Err(e) = self.process_cmd(sb.as_ref()) {
                            self.handle_error(e);
                        }
                    }
                    sb.truncate(0);
                    overflowed = false;
                    invalid = false;
                },
                _ => { invalid = true; }
            }
        }
    }

    /// Process one command received. Return whether we should exit
    fn process_cmd(&mut self, cmd: &str) -> Result<(),Error> {
        let mut iter = cmd.split(" ");

        match iter.next() {
            // open {base64 filename} {crc32}
            // Make 'filename' the currently open file. File will be opened
            // for read/write with the insertion point at the end.
            //Some("open") => self.do_open(&mut iter),
            Some("open") => self.data_cmd(&mut iter, FTrans::open_wrapped),

            // close
            // Close the currently open file.
            Some("close") => self.do_close(&mut iter),

            // read
            // Read 512 bytes (or whatever is left), moving the insertion point.
            // Returns as:
            //  ack {base64 data} {crc32}
            Some("read") => self.do_read(&mut iter),

            // write {base64 data} {crc32}
            // Write up to 512 bytes, moving the insertion point.
            // Returns as:
            //  ack
            //  error nack
            Some("write") => self.data_cmd(&mut iter, FTrans::write_wrapped),

            // truncate {file size as u32 LE, base64} {crc32}
            // Truncate the file to the given size
            // Returns as:
            //  ack
            //  error nack
            Some("truncate") => self.data_cmd(&mut iter, FTrans::truncate_wrapped),

            // seekset {position as u32 LE, base64} {crc32}
            // Set position relative to zero
            Some("seekset") => self.data_cmd(
                &mut iter,
                |s, i| {FTrans::seek_wrapped(s, i, ext4::Origin::Set)}),

            // seekcur {position as u32 LE, base64} {crc32}
            // Set position relative to current point
            Some("seekcur") => self.data_cmd(
                &mut iter,
                |s, i| {FTrans::seek_wrapped(s, i, ext4::Origin::Current)}),

            _ => {self.handle_invalid(); Ok(())},
        }
    }

    fn data_cmd<'a, I, F>(&mut self, iter: &'a mut I, f: F) -> Result<(), Error>
            where
                I: Iterator<Item=&'a str>,
                F: Fn(&mut Self, &[u8]) -> Result<(), Error>
    {
        let mut decode_buf = [0u8; 512];

        let data_b64 = try!(iter.next().ok_or(ERR_EXPECTED_ARGS));
        let n_bytes = try!(base64::decode(&mut decode_buf, data_b64.as_bytes()));

        let rx_crc32_str = try!(iter.next().ok_or(ERR_EXPECTED_ARGS));
        let rx_crc32 = try!(u32::parseint(rx_crc32_str));

        let actual_crc32 = crc32(&decode_buf[0..n_bytes]);

        if actual_crc32 != rx_crc32 {
            Err(ERR_CKSUM)
        } else {
            f(self, &decode_buf[0..n_bytes])
        }
    }

    fn open_wrapped(&mut self, filename: &[u8]) -> Result<(), Error>
    {
        let strslice = str::from_utf8(filename).unwrap();
        self.file = Some(try!(ext4::fopen(strslice, ext4::OpenFlags::ReadAppend)));

        println_async!("ack");
        Ok(())
    }

    fn write_wrapped(&mut self, data: &[u8]) -> Result<(), Error>
    {
        match self.file {
            Some(ref mut file) => {
                try!(file.write(data));
                println_async!("ack");
                Ok(())
            },
            None => {
                Err(ERR_FILE_NOT_OPEN)
            }
        }
    }

    fn do_close<'a, I>(&mut self, _iter: &'a mut I) -> Result<(), Error>
            where I: Iterator<Item=&'a str>
    {
        let was_open = self.file.is_some();

        self.file = None;

        if was_open {
            println_async!("ack");
        } else {
            println_async!("warn was_not_open");
        }

        Ok(())
    }

    fn do_read<'a, I>(&mut self, _iter: &'a mut I) -> Result<(), Error>
            where I: Iterator<Item=&'a str>
    {
        let mut file_buf = [0u8; 512];
        let mut b64_buf = [0u8; 700];

        match self.file {
            Some(ref mut file) => {
                let bytes_read = try!(file.read(&mut file_buf));
                let b64_converted = base64::encode(
                    &mut b64_buf, &file_buf[0..bytes_read]).unwrap();

                print_async!("ack ");
                for i in &b64_buf[0..b64_converted] {
                    print_async!("{}", *i as char);
                }
                println_async!("");

                Ok(())
            },
            None => Err(ERR_FILE_NOT_OPEN)
        }
    }

    fn truncate_wrapped(&mut self, sz_encoded: &[u8]) -> Result<(), Error>
    {
        let sz = try!(bytes_to_u32(sz_encoded));

        match self.file {
            Some(ref mut file) => {
                try!(file.truncate(sz as usize));
                println_async!("ack");
                Ok(())
            },
            None => {
                Err(ERR_FILE_NOT_OPEN)
            }
        }
    }

    fn seek_wrapped(&mut self, pos_encoded: &[u8], origin: ext4::Origin)
            -> Result<(), Error>
    {
        let pos = try!(bytes_to_u32(pos_encoded));

        match self.file {
            Some(ref mut file) => {
                try!(file.seek(pos as usize, origin));
                println_async!("ack");
                Ok(())
            },
            None => {
                Err(ERR_FILE_NOT_OPEN)
            }
        }
    }

    fn handle_overflow(&self) {
        println_async!("error overflow");
    }

    fn handle_invalid(&self) {
        println_async!("error invalid_byte_or_command");
    }

    fn handle_error(&self, e: Error) {
        println_async!("error {}", e);
    }
}

fn crc32(data: &[u8]) -> u32 {
    let len = data.len();
    !unsafe{
        lwext4_crc32::ext4_crc32(
            0xffffffff,
            data as *const _ as *const ctypes::c_void,
            len as u32)}
}

fn bytes_to_u32(data: &[u8]) -> Result<u32, Error>
{
    if data.len() != 4 {
        Err(ERR_ARG_RANGE)
    } else {
        Ok(
            (data[0] as u32) |
            ((data[1] as u32) << 8) |
            ((data[2] as u32) << 16) |
            ((data[3] as u32) << 24))
    }
}
