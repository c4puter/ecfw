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
FREERTOS = FreeRTOS

RUSTLIBS = core alloc
RUSTLIB_FILES = $(patsubst %,${RUSTLIB_DIR}/lib%.rlib,${RUSTLIBS})

LIBALLOC = rustsys/liballoc_system.rlib

RUST_CRATES = \
	libecfw_rust.rlib \

SUPPORT_CRATES = \
	libctypes.rlib \
	esh/esh_rust/src/libesh.rlib \

BINDGEN_CRATES = \
	hardware/libbindgen_mcu.rlib \
	hardware/libbindgen_usart.rlib \

ALL_CRATES = ${RUST_CRATES} ${SUPPORT_CRATES} ${BINDGEN_CRATES}

# Crates for which dependencies will be calculated. Do not include the
# bindgen crates here! That will result in bindgen being run at inappropriate
# times.
DEP_CRATES = ${RUST_CRATES} ${SUPPORT_CRATES}

OBJECTS = \
	hardware/mcu.o \
	hardware/usart.o \
	esh/esh_argparser.o \
	esh/esh.o \
	esh/esh_hist.o \
	${FREERTOS}/Source/queue.o \
	${FREERTOS}/Source/list.o \
	${FREERTOS}/Source/timers.o \
	${FREERTOS}/Source/tasks.o \
	${FREERTOS}/Source/croutine.o \
	${FREERTOS}/Source/event_groups.o \
	${FREERTOS}/Source/portable/MemMang/heap_1.o \
	${FREERTOS}/Source/portable/GCC/ARM_CM3/port.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.o \
	${ASF_UNF_DIR}/asf/drivers/pio/pio.o \
	${ASF_UNF_DIR}/asf/drivers/pmc/pmc.o \
	${ASF_UNF_DIR}/asf/drivers/usart/usart.o \
	${ASF_UNF_DIR}/asf/drivers/twi/twi.o \
	${ASF_UNF_DIR}/asf/services/clock/sam4s/sysclk.o \
	${ASF_UNF_DIR}/asf/utils/interrupt/interrupt_sam_nvic.o \

CFLAGS = \
	-Os -g -pipe -std=c99 -Wall -Wextra \
	-D__SAM4S16C__ -DARM_MATH_CM4=true -DBOARD=USER_BOARD \
	-mcpu=cortex-m4 -mthumb \
	-fdata-sections -ffunction-sections \
	-iquote config \
	-isystem ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/include \
	-isystem ${ASF_UNF_DIR}/asf/thirdparty/CMSIS/Include \
	-isystem ${FREERTOS}/Source/include \
	-isystem ${FREERTOS}/Source/portable/GCC/ARM_CM3 \
	-isystem ${ASF_UNF_DIR} \
	-I esh \

RUSTFLAGS = \
	-C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \
	-L ${RUSTLIB_DIR} -L . -L hardware -L esh/esh_rust/src

LDFLAGS = \
	-Wl,--entry=Reset_Handler \
	-mcpu=cortex-m4 -mthumb \
	-D__sam4s16c__ \
	-specs=nosys.specs \
	-Wl,--gc-sections \
	-Wl,-T,${ASF_UNF_DIR}/asf/utils/linker_scripts/sam4s/sam4s16/gcc/flash.ld \

LIBS = -lm -lc -lgcc -lnosys

.PHONY: all clean genclean distclean debug program
.SECONDARY: ${RUSTLIB_FILES}

-include ${OBJECTS:.o=.d}
-include $(patsubst %,%.d,${DEP_CRATES})

all: ecfw.hex ecfw.disasm
	${SIZE} ecfw

${RUST_CRATES}: ${SUPPORT_CRATES} ${BINDGEN_CRATES}

${BINDGEN_CRATES}: ${SUPPORT_CRATES}

lib%.rlib: %.rs ${RUSTLIB_FILES}
	@echo "[RUSTC rs] $@"
	@${RUSTC} ${RUSTFLAGS} --crate-type lib -o $@ $< && \
	if [[ "$@" =~ "${DEP_CRATES}" ]]; then \
	 	${RUSTC} ${RUSTFLAGS} --crate-type lib --emit dep-info -o $@.d $< 2>/dev/null ; \
	fi

lib%.rlib: % ${RUSTLIB_FILES}
	@echo "[RUSTC  /] $@"
	@${RUSTC} ${RUSTFLAGS} --crate-name=$$(basename $<) -o $@ $</lib.rs && \
	if [[ "$@" =~ "${DEP_CRATES}" ]]; then \
		${RUSTC} ${RUSTFLAGS} --crate-name=$$(basename $<) --emit dep-info -o $@.d $</lib.rs 2>/dev/null ; \
	fi

bindgen_%.rs: %.h have-bindgen
	@echo "[BINDGEN ] $@"
	@( echo '#![no_std]'; \
	  $$(cat have-bindgen) --use-core --convert-macros --ctypes-prefix=ctypes $< ) | \
	sed -e 's/)]$$/\0\nextern crate ctypes;/' \
	> $@

have-bindgen:
	@echo -n "[LOCATE  ] bindgen... "
	@( command -v bindgen >$@ && command -v bindgen | tee $@ ) || \
	( [ -x ${HOME}/.cargo/bin/bindgen ] && echo "${HOME}/.cargo/bin/bindgen" | tee $@ ) || \
	( echo -e "\n[INSTALL ] bindgen" && cargo install bindgen && \
			(( command -v bindgen >/dev/null 2>&1 && command -v bindgen > $@ ) || \
			 ( [ -x ${HOME}/.cargo/bin/bindgen ] && echo "${HOME}/.cargo/bin/bindgen" > $@ )))

${ASF_UNF_DIR}: ./scripts/unfuck-asf.py
	@if ! [ -e ${ASF_SOURCE} ]; then \
		echo ERROR - you must provide the Atmel ASF source, via either ASF_SOURCE= ; \
		echo or via a link or direct copy in resources/asf. For more information, ; \
		echo see README.md. ; \
		exit 1 ; \
	fi
	@echo "[UNFUCK  ] ${ASF_SOURCE}"
	@mkdir -p $@
	@cd $@; \
	${PYTHON} ../../scripts/unfuck-asf.py sam $(realpath ${ASF_SOURCE}) asf

${RUSTLIB_DIR}/lib%.rlib:
	@echo "[RUSTLIB ] $@"
	@bash ./scripts/build-rust-lib.sh $*

%.o: %.c ${ASF_UNF_DIR}
	@echo "[CC      ] $@"
	@${CC} -c  ${CFLAGS} $*.c -o $*.o
	@${CC} -MM ${CFLAGS} $*.c  > $*.d

ecfw: ${OBJECTS} ${ALL_CRATES}
	@echo "[CC LINK ] $@"
	@${CC} ${CFLAGS} ${LDFLAGS} ${LIBS} \
			${OBJECTS} ${ALL_CRATES} ${RUSTLIB_FILES} -o ecfw

ecfw.disasm: ecfw
	@echo "[OBJDUMP ] $@"
	@${OBJDUMP} -CS $< > $@

ecfw.hex: ecfw
	@echo "[OBJCOPY ] $@"
	@${OBJCOPY} -O ihex $< $@

clean:
	rm -f ${OBJECTS}
	rm -f ${ALL_CRATES}
	rm -f ecfw ecfw.hex ecfw.disasm
	rm -f ${OBJECTS:.o=.d}
	rm -f $(patsubst %,%.d,${DEP_CRATES})
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
