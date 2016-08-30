cmake_minimum_required(VERSION 3.0.2)
set(CMAKE_TOOLCHAIN_FILE CMakeModules/toolchain-gnu-arm-none-eabi-sam4s16c.cmake)
project(ecfw)

find_package(PythonInterp REQUIRED)

set(ECFW_ASF_SOURCE $ENV{HOME}/pantry/xdk-asf-3.31.0 CACHE PATH "Path to an unpacked Atmel ASF tree")
set(ASF_UNF_DIR ${CMAKE_BINARY_DIR}/asf-unf)

include_directories(SYSTEM
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/include
    ${ASF_UNF_DIR}/asf/thirdparty/CMSIS/Include
    ${ASF_UNF_DIR}
    )

include_directories(BEFORE
    config
    )

add_custom_target(asf-unf-dir
    COMMAND mkdir -p ${ASF_UNF_DIR}
    )

add_custom_target(asf-unf
    COMMAND ${PYTHON_EXECUTABLE} ${CMAKE_SOURCE_DIR}/unfuck-asf.py sam ${ECFW_ASF_SOURCE} asf
    WORKING_DIRECTORY ${ASF_UNF_DIR}
    DEPENDS asf-unf-dir
    )

add_executable( ecfw
    main.c
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.c
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.c
    ${ASF_UNF_DIR}/asf/drivers/pio/pio.c
    ${ASF_UNF_DIR}/asf/drivers/pmc/pmc.c
    ${ASF_UNF_DIR}/asf/services/clock/sam4s/sysclk.c
    )

set_source_files_properties(
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.c
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.c
    PROPERTIES GENERATED TRUE)

add_dependencies(ecfw asf-unf)

target_compile_options( ecfw PUBLIC
    -O1 -g -pipe
    -std=c99 -Wall -Wextra
    -D__SAM4S16C__ -DARM_MATH_CM4=true -DBOARD=USER_BOARD )

target_link_libraries( ecfw
    -Wl,--entry=Reset_Handler
    -Wl,--cref
    -mcpu=cortex-m4 -mthumb
    -D__sam4s16c__
    -specs=nosys.specs
    -Wl,--gc-sections
    -Wl,-T,${ASF_UNF_DIR}/asf/utils/linker_scripts/sam4s/sam4s16/gcc/flash.ld
    -Wl,-Map=flash.map,--cref
    -pipe

    m c gcc nosys )
