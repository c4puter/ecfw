CROSS_COMPILE ?= arm-none-eabi-
CC 		= ${CROSS_COMPILE}gcc
OBJCOPY	= ${CROSS_COMPILE}objcopy
OBJDUMP = ${CROSS_COMPILE}objdump
RUSTC   = rustc
PYTHON  ?= python

ASF_UNF_DIR = asf-unf
ASF_SOURCE ?= asf

LOCAL_OBJECTS = \
	main.o		\
	test.o		\

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
	-C opt-level=2 -Z no-landing-pads --target thumbv7em-none-eabi -g --emit obj -L libcore-thumbv7m

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

.PHONY: all clean distclean

%.o: %.rs libcore-thumbv7m
	${RUSTC} ${RUSTFLAGS} -o $@ $<

all: ecfw.hex

asf-unf: unfuck-asf.py
	mkdir -p $@
	cd $@; \
	${PYTHON} ../unfuck-asf.py sam $(realpath ${ASF_SOURCE}) asf

${ASF_UNF_DIR}/%: asf-unf

libcore-thumbv7m:
	bash ./build-rust-libcore.sh

ecfw: | asf-unf ${ASF_OBJECTS} ${LOCAL_OBJECTS}
	${CC} ${CFLAGS} ${LDFLAGS} ${ASF_OBJECTS} ${LOCAL_OBJECTS} ${LIBS} -o ecfw

ecfw.hex: ecfw
	${OBJCOPY} -O ihex $< $@

clean:
	rm -f ${ASF_OBJECTS}
	rm -f ${LOCAL_OBJECTS}
	rm -f flash.map
	rm -f ecfw ecfw.hex

distclean: clean
	rm -rf ${ASF_UNF_DIR}
	rm -rf libcore-thumbv7m
	rm -rf rustsrc
