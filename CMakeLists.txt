cmake_minimum_required(VERSION 3.0.2)
set(CMAKE_TOOLCHAIN_FILE CMakeModules/toolchain-gnu-arm-none-eabi-sam4s16c.cmake)
project(ecfw)

find_package(PythonInterp)

set(ECFW_ASF_SOURCE $ENV{HOME}/pantry/xdk-asf-3.31.0 CACHE PATH "Path to an unpacked Atmel ASF tree")
set(ASF_UNF_DIR ${CMAKE_BINARY_DIR}/asf-unf)

include_directories(SYSTEM
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/include
    ${ASF_UNF_DIR}/asf/thirdparty/CMSIS/Include
    ${ASF_UNF_DIR}
    )

add_custom_target(asf-unf
    COMMAND mkdir -p ${ASF_UNF_DIR}
    WORKING_DIRECTORY ${ASF_UNF_DIR}
    COMMAND ${PYTHON_EXECUTABLE} ${CMAKE_SOURCE_DIR}/unfuck-asf.py sam ${ECFW_ASF_SOURCE} asf
    )

add_executable( ecfw
    main.c
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.c
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.c
    )

set_source_files_properties(
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/system_sam4s.c
    ${ASF_UNF_DIR}/asf/utils/cmsis/sam4s/source/templates/gcc/startup_sam4s.c
    PROPERTIES GENERATED TRUE)

add_dependencies(ecfw asf-unf)

target_compile_options( ecfw PUBLIC
    -Og -g -pipe -nostartfiles
    -std=c99 -Wall -Wextra
    -D__SAM4S16C__ -DARM_MATH_CM4=true )

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
