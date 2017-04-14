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
}

/// Table of all error messages.
error_table! {
    ///////////////////////////////////////////////////////////////////
    // General, data processing
    ERR_UNKNOWN:                "unknown error";
    ERR_UTF8:                   "invalid UTF-8";
    ERR_UTF16_ORPHAN:           "orphaned UTF-16 surrogate";
    ERR_CODEPOINT:              "invalid Unicode codepoint";
    ERR_STRLEN:                 "string too long";

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

    ///////////////////////////////////////////////////////////////////
    // TWI_related
    ERR_TWI_INVALID:            "TWI: invalid argument";
    ERR_TWI_ARBITRATION:        "TWI: arbitration lost";
    ERR_TWI_NOTFOUND:           "TWI: chip not found";
    ERR_TWI_RXOVF:              "TWI: receive overrun";
    ERR_TWI_RXNACK:             "TWI: receive NACK";
    ERR_TWI_TXOVF:              "TWI: transmit overrun";
    ERR_TWI_TXNACK:             "TWI: transmit NACK";
}
