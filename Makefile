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

ASF_UNF_DIR = resources/asf-unf
ASF_SOURCE ?= resources/asf
RUSTLIB_DIR ?= resources/rustlibs
FREERTOS = ${ASF_UNF_DIR}/asf/thirdparty/freertos/freertos-8.2.3

RUSTLIBS = core alloc
RUSTLIB_FILES = $(patsubst %,${RUSTLIB_DIR}/lib%.rlib,${RUSTLIBS})

LOCAL_OBJECTS = \
	main/main.o \
	hardware/mcu.o \
	hardware/usart.o \

SUPPORT_CRATES = \
	rustsys/librust_support.rlib \

RUST_CRATES = \
	rustsys/liballoc_system.rlib \
	rustsys/libec_io.rlib \
	rustsys/libctypes.rlib \
	rustsys/libfreertos.rlib \

FREERTOS_OBJECTS = \
	${FREERTOS}/Source/queue.o \
	${FREERTOS}/Source/list.o \
	${FREERTOS}/Source/timers.o \
	${FREERTOS}/Source/tasks.o \
	${FREERTOS}/Source/croutine.o \
	${FREERTOS}/Source/event_groups.o \
	${FREERTOS}/Source/portable/MemMang/heap_1.o \
	freertos-port/port.o \

ASF_OBJECTS = \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.o \
	${ASF_UNF_DIR}/asf/drivers/pio/pio.o \
	${ASF_UNF_DIR}/asf/drivers/pmc/pmc.o \
	${ASF_UNF_DIR}/asf/drivers/usart/usart.o \
	${ASF_UNF_DIR}/asf/services/clock/sam4s/sysclk.o \
	${ASF_UNF_DIR}/asf/utils/interrupt/interrupt_sam_nvic.o \
	${FREERTOS_OBJECTS} \

CFLAGS = \
	-O1 -g -pipe -std=c99 -Wall -Wextra \
	-D__SAM4S16C__ -DARM_MATH_CM4=true -DBOARD=USER_BOARD \
	-mcpu=cortex-m4 -mthumb -mlong-calls \
	-fdata-sections -ffunction-sections \
	-iquote config \
	-isystem ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/include \
	-isystem ${ASF_UNF_DIR}/asf/thirdparty/CMSIS/Include \
	-isystem ${FREERTOS}/Source/include \
	-isystem freertos-port \
	-isystem ${ASF_UNF_DIR} \

RUSTFLAGS = \
	-C opt-level=0 -Z no-landing-pads --target thumbv7em-none-eabi -g \
	-L ${RUSTLIB_DIR} -L main -L hardware -L rustsys

	#-C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \

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

.PHONY: all clean genclean distclean debug program
.SECONDARY: ${RUSTLIB_FILES}

all: ecfw.hex
	${SIZE} ecfw

%.o: %.rs ${RUSTLIB_FILES} ${RUST_CRATES}
	${RUSTC} ${RUSTFLAGS} --crate-type staticlib --emit llvm-ir -o $(patsubst %.o,%.ll,$@) $<
	${RUSTC} ${RUSTFLAGS} --crate-type staticlib --emit obj -o $@ $<

lib%.rlib: %.rs ${RUSTLIB_FILES} ${LIBALLOC}
	${RUSTC} ${RUSTFLAGS} --crate-type lib --emit llvm-ir -o $(patsubst %.rlib,%.ll,$@) $<
	${RUSTC} ${RUSTFLAGS} --crate-type lib -o $@ $<

bindgen_%.rs: %.h have-bindgen
	echo bindgen > $@
	@( echo '#![no_std]'; \
	  $$(cat have-bindgen) --use-core --convert-macros --ctypes-prefix=ctypes $< ) | \
	sed -e 's/)]$$/\0\nextern crate ctypes;/' \
	> $@

-include deps.rust
-include ${LOCAL_OBJECTS:.o=.d}
-include ${ASF_OBJECTS:.o=.d}

deps.rust:
	bash ./scripts/gen-rust-dependencies.sh > $@

have-bindgen:
	@( command -v bindgen >$@ && command -v bindgen > $@ && echo "Found bindgen on path" ) || \
	( [ -x ${HOME}/.cargo/bin/bindgen ] && echo "${HOME}/.cargo/bin/bindgen" > $@ && echo "Found bindgen in ~/.cargo" ) || \
	( echo "Installing bindgen..." && cargo install bindgen && \
			(( command -v bindgen >/dev/null 2>&1 && command -v bindgen > $@ ) || \
			 ( [ -x ${HOME}/.cargo/bin/bindgen ] && echo "${HOME}/.cargo/bin/bindgen" > $@ )))

${ASF_UNF_DIR}: ./scripts/unfuck-asf.py
	@if ! [ -e ${ASF_SOURCE} ]; then \
		echo ERROR - you must provide the Atmel ASF source, via either ASF_SOURCE= ; \
		echo or via a link or direct copy in resources/asf. For more information, ; \
		echo see README.md. ; \
		exit 1 ; \
	fi
	mkdir -p $@
	cd $@; \
	${PYTHON} ../../scripts/unfuck-asf.py sam $(realpath ${ASF_SOURCE}) asf

${RUSTLIB_DIR}/lib%.rlib:
	bash ./scripts/build-rust-lib.sh $*

%.o: %.c ${ASF_UNF_DIR}
	${CC} -c  ${CFLAGS} $*.c -o $*.o
	${CC} -MM ${CFLAGS} $*.c  > $*.d

ecfw: ${LOCAL_OBJECTS} ${ASF_OBJECTS} ${RUST_CRATES} ${SUPPORT_CRATES}
	${CC} ${CFLAGS} ${LDFLAGS} ${LIBS} \
			${LOCAL_OBJECTS} ${ASF_OBJECTS} ${RUST_CRATES} \
			${RUSTLIB_FILES} ${SUPPORT_CRATES} -o ecfw

ecfw.hex: ecfw
	${OBJCOPY} -O ihex $< $@

clean:
	rm -f ${ASF_OBJECTS}
	rm -f ${LOCAL_OBJECTS}
	rm -f ${RUST_CRATES}
	rm -f ${SUPPORT_CRATES}
	rm -f $(patsubst %.o,%.ll,${LOCAL_OBJECTS})
	rm -f $(patsubst %.rlib,%.ll,${RUST_CRATES})
	rm -f $(patsubst %.rlib,%.ll,${SUPPORT_CRATES})
	rm -f flash.map
	rm -f ecfw ecfw.hex
	rm -f deps.rust
	rm -f ${LOCAL_OBJECTS:.o=.d}
	rm -f ${ASF_OBJECTS:.o=.d}
	rm -f have-bindgen

genclean: clean
	rm -rf ${ASF_UNF_DIR}
	rm -rf ${RUSTLIB_FILES}

distclean: genclean
	rm -rf resources/rustsrc

debug: ecfw
	bash ./scripts/debug

program: ecfw
	bash ./scripts/program
