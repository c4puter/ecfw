#/bin/bash


# rust lib builder
# Copyright (C) 2017 Chris Pavlina
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 2 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along
# with this program; if not, write to the Free Software Foundation, Inc.,
# 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

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
    echo "> rustup component add rust-src" >&2
    exit 1
fi

mkdir -p $RUSTLIB_DIR
pushd $RUSTSRC >/dev/null
mkdir -p lib_out
rustc -C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \
    --crate-type rlib \
    --crate-name "$1" \
    -L "$RUSTLIB_DIR_ABS" \
    lib$1/lib.rs --out-dir "$RUSTLIB_DIR_ABS"

    #lib$1/lib.rs --out-dir lib_out
popd >/dev/null

#mv $RUSTSRC/lib_out/lib$1.rlib $RUSTLIB_DIR/
