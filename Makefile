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

CROSS_COMPILE ?= arm-none-eabi-
CC 		= ${CROSS_COMPILE}gcc
OBJCOPY	= ${CROSS_COMPILE}objcopy
OBJDUMP = ${CROSS_COMPILE}objdump
SIZE    = ${CROSS_COMPILE}size
RUSTC   = rustc
PYTHON  ?= python

ASF_UNF_DIR = asf-unf
ASF_SOURCE ?= asf

LOCAL_OBJECTS = \
	main/main.o \
	hardware/mcu.o \

RUST_CRATES = \
	main/librust_support.rlib \

ASF_OBJECTS = \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.o \
	${ASF_UNF_DIR}/asf/drivers/pio/pio.o \
	${ASF_UNF_DIR}/asf/drivers/pmc/pmc.o \
	${ASF_UNF_DIR}/asf/services/clock/sam4s/sysclk.o \

CFLAGS = \
	-O1 -g -pipe -std=c99 -Wall -Wextra \
	-D__SAM4S16C__ -DARM_MATH_CM4=true -DBOARD=USER_BOARD \
	-mcpu=cortex-m4 -mthumb -mlong-calls \
	-fdata-sections -ffunction-sections \
	-I config \
	-isystem ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/include \
	-isystem ${ASF_UNF_DIR}/asf/thirdparty/CMSIS/Include \
	-isystem ${ASF_UNF_DIR} \

RUSTFLAGS = \
	-C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \
	-L libcore-thumbv7m -L main -L hardware

LDFLAGS = \
	-Wl,--entry=Reset_Handler \
	-Wl,--cref \
	-mcpu=cortex-m4 -mthumb \
	-D__sam4s16c__ \
	-specs=nosys.specs \
	-Wl,--gc-sections \
	-Wl,-T,${ASF_UNF_DIR}/asf/utils/linker_scripts/sam4s/sam4s16/gcc/flash.ld \
	-Wl,-Map=flash.map,--cref \

LIBS = -lm -lc -lgcc -lnosys

.PHONY: all clean genclean distclean

%.o: %.rs libcore-thumbv7m ${RUST_CRATES}
	${RUSTC} ${RUSTFLAGS} --crate-type staticlib --emit llvm-ir -o $(patsubst %.o,%.ll,$@) $<
	${RUSTC} ${RUSTFLAGS} --crate-type staticlib --emit obj -o $@ $<

lib%.rlib: %.rs libcore-thumbv7m
	${RUSTC} ${RUSTFLAGS} --crate-type lib --emit llvm-ir -o $(patsubst %.rlib,%.ll,$@) $<
	${RUSTC} ${RUSTFLAGS} --crate-type lib -o $@ $<

all: ecfw.hex
	${SIZE} ecfw

asf-unf: unfuck-asf.py
	mkdir -p $@
	cd $@; \
	${PYTHON} ../unfuck-asf.py sam $(realpath ${ASF_SOURCE}) asf

libcore-thumbv7m:
	bash ./build-rust-libcore.sh

%.o: %.c asf-unf
	${CC} ${CFLAGS} -c $< -o $@

ecfw: ${LOCAL_OBJECTS} ${ASF_OBJECTS} ${RUST_CRATES}
	${CC} ${CFLAGS} ${LDFLAGS} $^ ${LIBS} -o ecfw

ecfw.hex: ecfw
	${OBJCOPY} -O ihex $< $@

clean:
	rm -f ${ASF_OBJECTS}
	rm -f ${LOCAL_OBJECTS}
	rm -f ${RUST_CRATES}
	rm -f $(patsubst %.o,%.ll,${LOCAL_OBJECTS})
	rm -f $(patsubst %.rlib,%.ll,${RUST_CRATES})
	rm -f flash.map
	rm -f ecfw ecfw.hex

genclean: clean
	rm -rf ${ASF_UNF_DIR}
	rm -rf libcore-thumbv7m

distclean: genclean
	rm -rf rustsrc
