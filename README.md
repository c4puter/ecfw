C4-0 EC firmware
================

This is the firmware for the C4-0 embedded controller.
It's a bit of a beast to build, using both a modified Atmel ASF library and the
Rust programming language.

To build it, just use `make` — but first you'll have to acquire a few things:

ARM GCC toolchain
-----------------

The usual packages, typically called `arm-none-each-gcc`,
`arm-none-eabi-binutils`, and `arm-none-eabi-newlib`.


Atmel ASF
---------

Atmel's ASF support library looks open-source at first, but the extra term they
added to their license is a stinker, and I can't in good faith redistribute it.
Therefore, you will need to download a copy of ASF to build:

http://www.atmel.com/tools/avrsoftwareframework.aspx

There are two ways to give it to the build system. If you don't want to touch
anything in the source tree, put the unzipped ASF tree somewhere on your
system, and direct make to it with the `ASF_SOURCE` option.

Alternatively (this is what I do), either put it directly in the source tree
as a directory named `asf`, or symlink it into the source tree as `asf`
(preferred). This name is in `.gitignore` and so will not be accidentally
committed.

Rust compiler
-------------

In order to build Rust sources, a Rust compiler *and matching libcore* are
required.  The build system can make libcore itself, but it requires a Rust
compiler that knows which source revision it came from. In other words,
`rustc -v --version` must output a commit hash. If it does not, you may need
to build rustc from source. On Arch Linux, this is available in the Arch User
Repository as aur/rustc-git.

Be warned that the entire Rust source repository will be cloned by the build
system in order to check out the correct revision and build libcore. This is
big (a few hundred MB). If you keep the created directory `libcore-thumbv7m`
around, however, this will not be repeated. Do not `make distclean` unless
you want to download it again.
