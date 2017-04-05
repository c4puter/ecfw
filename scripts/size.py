#!/usr/bin/python

# Usage:
# size.py firmware.elf STACK_SIZE HEAP_SIZE TOTAL_FLASH TOTAL_SRAM

import subprocess
import sys

class colors:
    ENDC = '\033[0m'
    RED = '\033[31m'
    GREEN = '\033[32m'
    YELLOW = '\033[33m'
    BLUE = '\033[34m'
    MAGENTA = '\033[35m'

def main(argv):
    fw = argv[1]
    stack_size = int(argv[2], 0)
    heap_size = int(argv[3], 0)
    total_flash = int(argv[4], 0)
    total_sram = int(argv[5], 0)

    size_output = subprocess.check_output(["arm-none-eabi-size", fw])

    text_s, data_s, bss_s, dec_s, hex_s, filename_s = size_output.split(b'\n')[1].split()

    text = int(text_s)
    data = int(data_s)
    bss = int(bss_s)

    flash_consumed = text + data
    sram_consumed = data + bss

    # Bar chart
    #
    # Flash: ttttttttttttttddd_____________________________________
    # SRAM:  dddddddbbsssssshhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh___
    #
    # t: text   d: data     b: bss-stack-heap   s: stack    h: heap

    bar_width = 62
    flash_per_box = total_flash/bar_width
    sram_per_box = total_sram/bar_width

    flash_text_boxes = text // flash_per_box
    flash_data_boxes = data // flash_per_box
    flash_blank_boxes = bar_width - flash_text_boxes - flash_data_boxes

    sram_data_boxes = data // sram_per_box
    sram_bss_boxes = (bss - stack_size - heap_size) // sram_per_box
    sram_stack_boxes = stack_size // sram_per_box
    sram_heap_boxes = heap_size // sram_per_box
    sram_blank_boxes = bar_width - sram_data_boxes - sram_bss_boxes - sram_stack_boxes - sram_heap_boxes

    print("#" * 72)
    print("# %6.1f kiB (%3d%%) flash" % (flash_consumed / 1024, 100 * flash_consumed / total_flash))
    print("# %6.1f kiB (%3d%%)   text" % (text / 1024, 100 * text / total_flash))
    print("# %6.1f kiB (%3d%%)   data" % (data / 1024, 100 * data / total_flash))
    print("#" + "-" * 71)
    print("# %6.1f kiB (%3d%%) RAM" % (sram_consumed / 1024, 100 * sram_consumed / total_sram))
    print("# %6.1f kiB (%3d%%)   data" % (data / 1024, 100 * data / total_sram))
    print("# %6.1f kiB (%3d%%)   bss" % (bss / 1024, 100 * bss / total_sram))
    print("# %6.1f kiB (%3d%%)     stack, reserved" % (stack_size / 1024, 100 * stack_size / total_sram))
    print("# %6.1f kiB (%3d%%)     heap, reserved" % (heap_size / 1024, 100 * heap_size / total_sram))
    print("#" * 72)
    print("#")

    print("# Flash: ", end='')
    print(colors.RED + "t" * int(flash_text_boxes), end='')
    print(colors.GREEN + "d" * int(flash_data_boxes), end='')
    print(colors.ENDC + "_" * int(flash_blank_boxes))

    print("# SRAM:  ", end='')
    print(colors.GREEN + "d" * int(sram_data_boxes), end='')
    print(colors.YELLOW + "b" * int(sram_bss_boxes), end='')
    print(colors.BLUE + "s" * int(sram_stack_boxes), end='')
    print(colors.MAGENTA + "h" * int(sram_heap_boxes), end='')
    print(colors.ENDC + "_" * int(sram_blank_boxes))

    print("#")
    print("# ", end='')
    print(colors.RED + "t:" + colors.ENDC + " text     ", end='')
    print(colors.GREEN + "d:" + colors.ENDC + " data     ", end='')
    print(colors.YELLOW + "b:" + colors.ENDC + " bss  ( with  ", end='')
    print(colors.BLUE + "s:" + colors.ENDC + " stack   and   ", end='')
    print(colors.MAGENTA + "h:" + colors.ENDC + " heap  )")

    print("#")
    print("#" * 72)

if __name__ == "__main__":
    main(sys.argv)
