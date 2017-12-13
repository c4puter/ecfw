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

//! Error messages and debug categories

pub use rustsys::debug::*;

/// Table of all debug message categories.
debug_table! {

    // Category name
    // |                Message prefix
    // |                |           Enabled by default
    DEBUG_SYSMAN:       "sysman",   true;
    DEBUG_PWRBTN:       "pwrbtn",   false;
    DEBUG_ECBOOT:       "ecboot",   true;
    DEBUG_RESET:        "reset",    true;
    DEBUG_FS:           "ext3fs",   true;
    DEBUG_ALLOC:        "alloc",    false;
    DEBUG_CLOCK:        "clock",    true;
    DEBUG_SDRAM:        "sdram",    true;
}

/// Table of all error messages.
error_table! {
    ///////////////////////////////////////////////////////////////////
    // General, data processing
    ERR_UNKNOWN:                "unknown error";
    ERR_UTF8:                   "invalid UTF-8";
    ERR_UTF16_ORPHAN:           "orphaned UTF-16 surrogate";
    ERR_CODEPOINT:              "invalid Unicode codepoint";
    ERR_BASE64:                 "invalid base64 data";
    ERR_STRLEN:                 "string too long";
    ERR_DIGIT:                  "digit invalid or out of range for radix";
    ERR_NRANGE:                 "number out of range";
    ERR_CKSUM:                  "invalid checksum";

    // Command-related
    ERR_CANNOT_FIND:            "cannot find specified item";
    ERR_EXPECTED_ARGS:          "expected argument(s)";
    ERR_TOO_MANY_ARGS:          "too many arguments";
    ERR_ARG_RANGE:              "argument out of range";
    ERR_PARSE_ARGUMENT:         "cannot parse argument";
    ERR_RESET_FAILED:           "did not reset";

    ///////////////////////////////////////////////////////////////////
    // Errno
    ERR_EPERM:                  "operation not permitted";
    ERR_ENOENT:                 "no such file or directory";
    ERR_EIO:                    "I/O error";
    ERR_ENXIO:                  "no such device or address";
    ERR_E2BIG:                  "argument list too long";
    ERR_ENOMEM:                 "out of memory";
    ERR_EACCES:                 "permission denied";
    ERR_EFAULT:                 "bad address";
    ERR_EEXIST:                 "file exists";
    ERR_ENODEV:                 "no such device";
    ERR_ENOTDIR:                "not a directory";
    ERR_EISDIR:                 "is a directory";
    ERR_EINVAL:                 "invalid argument";
    ERR_EFBIG:                  "file too large";
    ERR_ENOSPC:                 "no space left on device";
    ERR_EROFS:                  "read-only file system";
    ERR_EMLINK:                 "too many links";
    ERR_ERANGE:                 "math result not representable";
    ERR_ENOTEMPTY:              "directory not empty";
    ERR_ENODATA:                "no data available";
    ERR_ENOTSUP:                "not supported";

    ///////////////////////////////////////////////////////////////////
    // General IO-related
    ERR_BUSY:                   "busy";
    ERR_TIMEOUT:                "timeout";

    ///////////////////////////////////////////////////////////////////
    // Disk-related
    ERR_NO_CARD:                "SD: card not found";
    ERR_SD_INIT_ONGOING:        "SD: init ongoing";
    ERR_SD_UNUSABLE:            "SD: card unusable";
    ERR_SD_SLOT:                "SD: invalid slot number";
    ERR_SD_COMM:                "SD: communications error";
    ERR_SD_PARAM:               "SD: invalid argument";
    ERR_SD_WRITE_PROT:          "SD: card is write protected";

    ///////////////////////////////////////////////////////////////////
    // Partition/FS-related
    ERR_GPT_SIGNATURE:          "GPT: invalid signature";
    ERR_GPT_ZEROLEN:            "GPT: zero entry length";
    ERR_GPT_SIZEMULT:           "GPT: block size must be multiple of entry length";
    ERR_NO_BOOT_PART:           "no boot parition found";
    ERR_FILE_NOT_OPEN:          "file not open";

    ///////////////////////////////////////////////////////////////////
    // I2C-related
    ERR_I2C_INVALID:            "I2C: invalid argument";
    ERR_I2C_ARBITRATION:        "I2C: arbitration lost";
    ERR_I2C_NOTFOUND:           "I2C: chip not found";
    ERR_I2C_RXOVF:              "I2C: receive overrun";
    ERR_I2C_RXNACK:             "I2C: receive NACK";
    ERR_I2C_TXOVF:              "I2C: transmit overrun";
    ERR_I2C_TXNACK:             "I2C: transmit NACK";

    ///////////////////////////////////////////////////////////////////
    // Oddly specific
    ERR_PLL_RANGE:              "PLL frequency out of range";
    ERR_CAS:                    "SDRAM: unsupported CAS latency";
}
