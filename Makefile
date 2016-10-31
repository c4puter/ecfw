# vim:foldmethod=marker:foldlevel=0

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

# TOOLCHAIN DEFINITIONS	{{{
###############################################################################

CROSS_COMPILE ?= arm-none-eabi-
CC 		= ${CROSS_COMPILE}gcc
OBJCOPY	= ${CROSS_COMPILE}objcopy
OBJDUMP = ${CROSS_COMPILE}objdump
SIZE    = ${CROSS_COMPILE}size
RUSTC   = rustc
PYTHON  ?= python

# }}}

# COMPILED OBJECTS {{{
###############################################################################

OBJECTS = \
	hardware/mcu.o \
	esh/esh_argparser.o \
	esh/esh.o \
	esh/esh_hist.o \
	FreeRTOS/Source/queue.o \
	FreeRTOS/Source/list.o \
	FreeRTOS/Source/timers.o \
	FreeRTOS/Source/tasks.o \
	FreeRTOS/Source/croutine.o \
	FreeRTOS/Source/event_groups.o \
	FreeRTOS/Source/portable/MemMang/heap_1.o \
	FreeRTOS/Source/portable/GCC/ARM_CM3/port.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.o \
	${ASF_UNF_DIR}/asf/drivers/pio/pio.o \
	${ASF_UNF_DIR}/asf/drivers/pmc/pmc.o \
	${ASF_UNF_DIR}/asf/drivers/rstc/rstc.o \
	${ASF_UNF_DIR}/asf/drivers/usart/usart.o \
	${ASF_UNF_DIR}/asf/drivers/twi/twi.o \
	${ASF_UNF_DIR}/asf/services/clock/sam4s/sysclk.o \
	${ASF_UNF_DIR}/asf/utils/interrupt/interrupt_sam_nvic.o \

RUSTLIBS = core alloc

RUST_CRATES = \
	libecfw_rust.rlib \

SUPPORT_CRATES = \
	libctypes.rlib \
	esh/esh_rust/src/libesh.rlib \
	hardware/libbindgen_mcu.rlib \
	${BINDGEN_CRATES}

RUST_PLUGINS = \
	plugins/librepeat.so \

# }}}

# GENERATED SOURCES {{{
###############################################################################

BINDGEN_SOURCES = \
	hardware/bindgen_mcu.rs:hardware/mcu.h \
	asf_usart.rs:${ASF_UNF_DIR}/asf/drivers/usart/usart.h \
	asf_rstc.rs:${ASF_UNF_DIR}/asf/drivers/rstc/rstc.h \

ASF_UNF_DIR = resources/asf-unf

# }}}

# BUILD FLAGS {{{
###############################################################################

CFLAGS = \
	-Os -g -pipe -std=c99 -Wall -Wextra \
	-D__SAM4S16C__ -DARM_MATH_CM4=true -DBOARD=USER_BOARD \
	-mcpu=cortex-m4 -mthumb \
	-fdata-sections -ffunction-sections \
	-iquote config \
	-isystem ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/include \
	-isystem ${ASF_UNF_DIR}/asf/thirdparty/CMSIS/Include \
	-isystem FreeRTOS/Source/include \
	-isystem FreeRTOS/Source/portable/GCC/ARM_CM3 \
	-isystem ${ASF_UNF_DIR} \
	-I esh \

RUSTFLAGS = \
	-C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g \
	-L ${RUSTLIB_DIR} -L . -L hardware -L esh/esh_rust/src -L plugins

LDFLAGS = \
	-Wl,--entry=Reset_Handler \
	-mcpu=cortex-m4 -mthumb \
	-D__sam4s16c__ \
	-specs=nosys.specs \
	-Wl,--gc-sections \
	-Wl,-T,${ASF_UNF_DIR}/asf/utils/linker_scripts/sam4s/sam4s16/gcc/flash.ld \

LIBS = -lm -lc -lgcc -lnosys

# }}}

# ENVIRONMENT VARIABLES {{{
###############################################################################

export GIT_HASH := $(shell git rev-parse --short HEAD 2>/dev/null || echo '(no git)')
export BUILD_ID := ${GIT_HASH}, $(shell date)

# }}}

# RESOURCES {{{

ASF_SOURCE ?= resources/asf
RUSTLIB_DIR ?= resources/rustlibs

# }}}

# FUNCTIONS AND COLLECTIONS {{{
###############################################################################

# Apply patsubst to just the file part of a path
# $(call filepatsubst,pattern,replacement,path)
filepatsubst = $(dir ${3})$(patsubst ${1},${2},$(notdir ${3}))

# Get the nth element of a :-separated list
# $(call nth,n,list)
# $(call nth,2,a:b:c)	-> b
nth = $(word ${1},$(subst :, ,${2}))

# All Rust crates to be linked into the final executable.
ALL_CRATES = ${RUST_CRATES} ${SUPPORT_CRATES}

# Crates built from Rust standard libs (libcore, liballoc, etc)
RUSTLIB_FILES = $(patsubst %,${RUSTLIB_DIR}/lib%.rlib,${RUSTLIBS})

# Crates built from bindgen-generated sources
BINDGEN_CRATES = $(foreach i,${BINDGEN_SOURCES}, \
				 $(call filepatsubst,%.rs,lib%.rlib,$(call nth,1,${i})))

# }}}

# COMMAND TARGETS {{{
###############################################################################

.PHONY: all all-with-asf clean genclean distclean debug program
.SECONDARY: ${RUSTLIB_FILES}

all: do-bindgen ${RUST_PLUGINS} ${ASF_UNF_DIR}
	${MAKE} all-with-asf

all-with-asf: ecfw.hex ecfw.disasm
	${SIZE} ecfw

clean:
	rm -f ${OBJECTS}
	rm -f ${ALL_CRATES}
	rm -f $(foreach i,${BINDGEN_SOURCES},$(word 1,$(subst :, ,${i})))
	rm -f ecfw ecfw.hex ecfw.disasm
	rm -f ${OBJECTS:.o=.d}
	rm -f $(patsubst %,%.d,${ALL_CRATES})
	rm -f ${RUST_PLUGINS}
	rm -f have-bindgen do-bindgen

genclean: clean
	rm -rf ${ASF_UNF_DIR}
	rm -rf ${RUSTLIB_FILES}

distclean: genclean
	rm -rf resources/rustsrc

debug: ecfw
	bash ./scripts/debug

program: ecfw
	bash ./scripts/program

# }}}

# INTERNAL COMMAND TARGETS {{{
###############################################################################

have-bindgen:
	@echo -n "[LOCATE  ] bindgen... "
	@( command -v bindgen >$@ && command -v bindgen | tee $@ ) || \
	( [ -x ${HOME}/.cargo/bin/bindgen ] && echo "${HOME}/.cargo/bin/bindgen" | tee $@ ) || \
	( echo -e "\n[INSTALL ] bindgen" && cargo install bindgen && \
			(( command -v bindgen >/dev/null 2>&1 && command -v bindgen > $@ ) || \
			 ( [ -x ${HOME}/.cargo/bin/bindgen ] && echo "${HOME}/.cargo/bin/bindgen" > $@ )))

define bindgen
	@echo "[BINDGEN ] $(2)"
	@( ( echo '#![no_std]'; \
		$$(cat have-bindgen) --use-core --convert-macros --builtins \
			--ctypes-prefix=ctypes --no-rust-enums $(1) -- \
			$(filter-out -mcpu=cortex-m4 -mthumb,${CFLAGS}) ) | \
		sed -e '0,/)]$$/ s//\0\n#![allow(improper_ctypes)]\nextern crate ctypes;/' \
		> $(2) ) 2>&1 | sed -e '/^WARN:bindgen/d' >&2

endef

do-bindgen: have-bindgen $(foreach i,${BINDGEN_SOURCES},$(call nth,2,${i}))
	$(foreach i,${BINDGEN_SOURCES}, \
		$(call bindgen,$(call nth,2,${i}),$(call nth,1,${i})))
	@touch do-bindgen

# }}}

# PATTERN TARGETS {{{
###############################################################################

lib%.rlib: %.rs ${RUSTLIB_FILES}
	@echo "[RUSTC rs] $@"
	@${RUSTC} ${RUSTFLAGS} --crate-type lib -o $@ $<
	@${RUSTC} ${RUSTFLAGS} --crate-type lib --emit dep-info -o $@.d $< 2>/dev/null
	@sed -i -e 's/\.rlib\.d:/\.rlib:/' $@.d

lib%.rlib: % ${RUSTLIB_FILES}
	@echo "[RUSTC  /] $@"
	@${RUSTC} ${RUSTFLAGS} --crate-name=$$(basename $<) -o $@ $</lib.rs
	@${RUSTC} ${RUSTFLAGS} --crate-name=$$(basename $<) --emit dep-info -o $@.d $</lib.rs 2>/dev/null
	@sed -i -e 's/\.rlib\.d:/\.rlib:/' $@.d

# rustc plugins
lib%.so: %.rs
	@echo "[RUSTC so] $@"
	@${RUSTC} --crate-type dylib -o $@ $<

%.o: %.c ${ASF_UNF_DIR}
	@echo "[CC      ] $@"
	@${CC} -c  ${CFLAGS} $*.c -o $*.o
	@${CC} -MM ${CFLAGS} $*.c  > $*.d

# }}}

# RESOURCE TARGETS {{{
###############################################################################

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

# }}}

# FINAL EXECUTABLE TARGETS {{{
###############################################################################
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

# }}}

# ADDITIONAL DEPENDENCIES {{{
###############################################################################

-include ${OBJECTS:.o=.d}
-include $(patsubst %,%.d,${ALL_CRATES})
${RUST_CRATES}: ${SUPPORT_CRATES}
${BINDGEN_CRATES}: libctypes.rlib

# }}}

