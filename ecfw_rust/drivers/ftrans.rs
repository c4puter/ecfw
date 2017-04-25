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

use data::{StringBuilder,base64};
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

    /// Open a file transfer session. Quits when either ^C, ^D, or a "quit"
    /// command is received.
    pub fn run(&mut self) {
        ec_io::flush_output();
        let mut sb = StringBuilder::new();
        let mut overflowed = false;
        let mut invalid = false;

        loop {
            let c = ec_io::getc_async();

            match c {
                3 | 4 => { /* ^C or ^D */ break; },
                0x20 ... 0x7e => { overflowed = sb.append_char(c as char).is_err(); },
                b'\r' => {
                    if overflowed {
                        self.handle_overflow();
                    } else if invalid {
                        self.handle_invalid();
                    } else {
                        match self.process_cmd(sb.as_ref()) {
                            Ok(true) => { break; },
                            Ok(false) => (),
                            Err(_) => { self.handle_invalid(); }
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
    fn process_cmd(&mut self, cmd: &str) -> Result<bool,Error> {
        let mut iter = cmd.split(" ");

        match iter.next() {
            Some("open") => self.do_open(&mut iter),
            Some("close") => self.do_close(&mut iter),
            Some("read") => self.do_read(&mut iter),
            Some("write") => self.do_write(&mut iter),
            Some("quit") => Ok(true),
            _ => {self.handle_invalid(); Ok(false)},
        }
    }

    fn do_open<'a, I>(&mut self, iter: &'a mut I) -> Result<bool,Error>
            where I: Iterator<Item=&'a str>
    {
        let filename_b64 = try!(iter.next().ok_or(ERR_EXPECTED_ARGS));

        let mut decode_buf = [0u8; 512];

        let written = try!(base64::decode(&mut decode_buf, filename_b64.as_bytes()));

        let slice = &decode_buf[0..written];
        let strslice = str::from_utf8(slice).unwrap();

        self.file = Some(try!(ext4::fopen(strslice, ext4::OpenFlags::ReadAppend)));

        println_async!("ack");
        Ok(false)
    }

    fn do_close<'a, I>(&mut self, _iter: &'a mut I) -> Result<bool,Error>
            where I: Iterator<Item=&'a str>
    {
        let was_open = self.file.is_some();

        self.file = None;

        if was_open {
            println_async!("ack");
        } else {
            println_async!("warn was_not_open");
        }

        Ok(false)
    }

    fn do_read<'a, I>(&mut self, _iter: &'a mut I) -> Result<bool,Error>
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

                Ok(false)
            },
            None => Err(ERR_FILE_NOT_OPEN)
        }
    }

    fn do_write<'a, I>(&mut self, iter: &'a mut I) -> Result<bool,Error>
            where I: Iterator<Item=&'a str>
    {
        let data_b64 = try!(iter.next().ok_or(ERR_EXPECTED_ARGS));
        let mut decode_buf = [0u8; 512];
        let written = try!(base64::decode(&mut decode_buf, data_b64.as_bytes()));

        let slice = &decode_buf[0..written];

        match self.file {
            Some(ref mut file) => {
                try!(file.write(slice));
                println_async!("ack");
                Ok(false)
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
}
