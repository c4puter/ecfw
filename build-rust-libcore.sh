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

if [[ -f libcore-thumbv7m/libcore.rlib ]]; then
    echo "rust libcore already built"
    exit 0
fi

RUST_COMMIT_HASH="$(rustc -v --version | grep commit-hash | awk '{print $2}')"

if [[ "$RUST_COMMIT_HASH" == "unknown" ]]; then
    echo "Cannot determine rustc commit hash." >&2
    echo "This is necessary to match to a revision of libcore." >&2
    echo "You may need to install rustc from source." >&2
    echo "On Arch Linux, this can be installed as aur/rust-git." >&2
    exit 1
fi

if ! [[ -e rustsrc/rust ]]; then
    mkdir -p rustsrc
    cd rustsrc

    git clone https://github.com/rust-lang/rust
    cd rust
    git checkout "$RUST_COMMIT_HASH"
    cd ..
    cp ../thumbv7em-none-eabi.json .
    cd ..
fi

cd rustsrc
mkdir -p libcore-thumbv7m
echo Compiling rust libcore...
rustc -C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \
    rust/src/libcore/lib.rs --out-dir libcore-thumbv7m

mv libcore-thumbv7m ../
