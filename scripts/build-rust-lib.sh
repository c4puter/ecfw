#/bin/bash

# The MIT License (MIT)
# Copyright (c) 2016 Chris Pavlina
# 
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
# 
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
# 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
# IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
# DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
# OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
# OR OTHER DEALINGS IN THE SOFTWARE.


set -e

RUSTLIB_DIR=resources/rustlibs
RUSTSRC=resources/rustsrc

# For linking
RUSTLIB_DIR_ABS="$PWD/$RUSTLIB_DIR"

if [[ -f "$RUSTLIB_DIR/lib$1.rlib" ]]; then
    echo "rust lib$1 already built"
    exit 0
fi

RUST_SYSROOT="$(rustc --print sysroot)"
RUSTSRC="$RUST_SYSROOT/lib/rustlib/src/rust/src"

if ! [[ -d "$RUSTSRC" ]]; then
    echo "rust-src component must be installed via rustup" >&2
    exit 1
fi

mkdir -p $RUSTLIB_DIR
pushd $RUSTSRC >/dev/null
mkdir -p lib_out
rustc -C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \
    -L "$RUSTLIB_DIR_ABS" \
    lib$1/lib.rs --out-dir "$RUSTLIB_DIR_ABS"

    #lib$1/lib.rs --out-dir lib_out
popd >/dev/null

#mv $RUSTSRC/lib_out/lib$1.rlib $RUSTLIB_DIR/
