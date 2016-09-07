#/bin/bash

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

if [[ -e rustsrc ]]; then
    echo "Deleting existing rust source directory"
    rm -rf rustsrc
fi

mkdir rustsrc
cd rustsrc

git clone https://github.com/rust-lang/rust
cd rust
git checkout "$RUST_COMMIT_HASH"
cd ..
cp ../thumbv7em-none-eabi.json .

mkdir -p libcore-thumbv7m
echo Compiling rust libcore...
rustc -C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \
    rust/src/libcore/lib.rs --out-dir libcore-thumbv7m

mv libcore-thumbv7m ../
