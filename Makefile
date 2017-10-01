# vim:foldmethod=marker:foldlevel=0

# c4puter embedded controller firmware
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
	FreeRTOS/Source/portable/MemMang/heap_4.o \
	FreeRTOS/Source/portable/GCC/ARM_CM3/port.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.o \
	${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.o \
	${ASF_UNF_DIR}/asf/drivers/pio/pio.o \
	${ASF_UNF_DIR}/asf/drivers/pmc/pmc.o \
	${ASF_UNF_DIR}/asf/drivers/rstc/rstc.o \
	${ASF_UNF_DIR}/asf/drivers/usart/usart.o \
	${ASF_UNF_DIR}/asf/drivers/twi/twi.o \
	${ASF_UNF_DIR}/asf/drivers/hsmci/hsmci.o \
	${ASF_UNF_DIR}/asf/components/memory/sd_mmc/sd_mmc.o \
	${ASF_UNF_DIR}/asf/services/clock/sam4s/sysclk.o \
	${ASF_UNF_DIR}/asf/services/delay/sam/cycle_counter.o \
	${ASF_UNF_DIR}/asf/utils/interrupt/interrupt_sam_nvic.o \
	lwext4/src/ext4_balloc.o \
	lwext4/src/ext4_bcache.o \
	lwext4/src/ext4_bitmap.o \
	lwext4/src/ext4_blockdev.o \
	lwext4/src/ext4_block_group.o \
	lwext4/src/ext4.o \
	lwext4/src/ext4_crc32.o \
	lwext4/src/ext4_debug.o \
	lwext4/src/ext4_dir.o \
	lwext4/src/ext4_dir_idx.o \
	lwext4/src/ext4_extent.o \
	lwext4/src/ext4_fs.o \
	lwext4/src/ext4_hash.o \
	lwext4/src/ext4_ialloc.o \
	lwext4/src/ext4_inode.o \
	lwext4/src/ext4_journal.o \
	lwext4/src/ext4_mbr.o \
	lwext4/src/ext4_mkfs.o \
	lwext4/src/ext4_super.o \
	lwext4/src/ext4_trans.o \
	lwext4/src/ext4_xattr.o \

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
	asf_hsmci.rs:${ASF_UNF_DIR}/asf/drivers/hsmci/hsmci.h \
	asf_sd_mmc.rs:${ASF_UNF_DIR}/asf/components/memory/sd_mmc/sd_mmc.h \
	lwext4_crc32.rs:lwext4/include/ext4_crc32.h \
	lwext4_blockdev.rs:lwext4/include/ext4_blockdev.h \
	lwext4.rs:lwext4/include/ext4.h \

ASF_UNF_DIR = resources/asf-unf

# }}}

# BUILD FLAGS {{{
###############################################################################

STACK_SIZE=0x400
HEAP_SIZE=0x1bc00
TOTAL_FLASH=0x100000
TOTAL_SRAM=0x20000

CFLAGS = \
	-Os -g -pipe -std=c99 -Wall -Wextra -Wno-int-conversion \
	-D__SAM4S16C__ -DARM_MATH_CM4=true -DBOARD=USER_BOARD \
	-D__HEAP_SIZE__=${HEAP_SIZE} \
	-DCONFIG_UNALIGNED_ACCESS=1 \
	-DCONFIG_DEBUG_PRINTF=0 \
	-DCONFIG_DEBUG_ASSERT=0 \
	-DCONFIG_USE_USER_MALLOC=1 \
	-DCONFIG_USE_DEFAULT_CFG=1 \
	-DCONFIG_HAVE_OWN_ERRNO=1 \
	-DCONFIG_HAVE_OWN_OFLAGS=1 \
	-mcpu=cortex-m4 -mthumb \
	-fdata-sections -ffunction-sections \
	-iquote config \
	-isystem lwext4/include \
	-isystem ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/include \
	-isystem ${ASF_UNF_DIR}/asf/thirdparty/CMSIS/Include \
	-isystem FreeRTOS/Source/include \
	-isystem FreeRTOS/Source/portable/GCC/ARM_CM3 \
	-isystem ${ASF_UNF_DIR} \
	-I esh \

RUSTFLAGS = \
	-C opt-level=1 -Z no-landing-pads --target thumbv7em-none-eabi -g \
	-L ${RUSTLIB_DIR} -L . -L hardware -L esh/esh_rust/src -L plugins

BINDGENFLAGS = \
	--use-core --builtins \
	--ctypes-prefix ::ctypes \
	--no-doc-comments \
	--raw-line '\#![no_std]' \
	--raw-line '\#![feature(untagged_unions)]' \
	--raw-line '\#![allow(improper_ctypes)]' \
	--raw-line '\#![allow(non_camel_case_types)]' \
	--raw-line '\#![allow(non_snake_case)]' \
	--raw-line '\#![allow(non_upper_case_globals)]' \
	--raw-line 'extern crate ctypes;' \
	--opaque-type 'ext4_bcache' \
	-- $(filter-out -mcpu=cortex-m4 -mthumb,${CFLAGS})


LDFLAGS = \
	-Wl,--entry=Reset_Handler \
	-mcpu=cortex-m4 -mthumb \
	-D__sam4s16c__ \
	-Wl,--defsym,__stack_size__=${STACK_SIZE} \
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

.PHONY: all all-with-asf clean genclean distclean debug program reset
.SECONDARY: ${RUSTLIB_FILES}

all: do-bindgen ${RUST_PLUGINS} ${ASF_UNF_DIR}
	${MAKE} all-with-asf

all-with-asf: ecfw.hex ecfw.disasm
	${PYTHON} scripts/size.py ecfw ${STACK_SIZE} ${HEAP_SIZE} ${TOTAL_FLASH} ${TOTAL_SRAM}

clean:
	rm -f ${OBJECTS}
	rm -f ${STATLIBS}
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

debug: ecfw
	bash ./scripts/control ecfw debug

program: ecfw
	bash ./scripts/control ecfw program

reset:
	bash ./scripts/control ecfw reset

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
	@( $$(cat have-bindgen) $(1) ${BINDGENFLAGS} \
		> $(2) ) 2>&1 | sed -e '/^WARN:bindgen/d' >&2

endef

do-bindgen: have-bindgen ${ASF_UNF_DIR} $(foreach i,${BINDGEN_SOURCES},$(call nth,2,${i}))
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

