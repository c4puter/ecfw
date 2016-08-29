cmake_minimum_required(VERSION 3.0.2)
project(ecfw)

include_directories(
    cm4-support/sam4s/include
    cm4-support/Include
    )

add_executable( ecfw
    main.c
    cm4-support/sam4s/source/templates/system_sam4s.c
    cm4-support/sam4s/source/templates/gcc/startup_sam4s.c
    )

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
    -Wl,-T,${CMAKE_SOURCE_DIR}/cm4-support/flash.ld
    -Wl,-Map=flash.map,--cref
    -pipe

    m c gcc nosys )
