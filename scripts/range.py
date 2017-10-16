# GDB plugin to find the PC ranges over which a variable is valid
# This is a hacky workaround for significant inlining making
# variables invisible, and this script is broken as shit. Use at your
# own risk.
#
# Usage:  range VARIABLE
# Prints a list of PC ranges.
#
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

import gdb
import subprocess
import os

def get_file_table(ofile):
    fulltab = {} # (stmt_list, filenum) => path
    dirtab = {}
    filetab = {}
    dump = subprocess.check_output(["arm-none-eabi-objdump", "--dwarf=rawline", "--", ofile]).decode("ascii")
    in_dirtab = False
    in_filetab = False
    offset = None

    for line in dump.split("\n"):
        if line.startswith("  Offset:"):
            for index, fn in filetab.items():
                fulltab[(offset,index)] = fn
            filetab = {}
            dirtab = {}
            offset = line.split(None, 1)[1]
        elif line.startswith(" The Directory Table"):
            in_dirtab = True
            in_filetab = False
            continue
        elif line.startswith(" The File Name Table"):
            in_dirtab = False
            in_filetab = True
            continue

        if in_dirtab:
            try:
                index, name = line.split(None, 1)
            except ValueError:
                pass
            else:
                dirtab[index] = name
        elif in_filetab:
            try:
                index, dirindex, time, size, name = line.split(None, 4)
            except ValueError:
                pass
            else:
                try:
                    filetab[index] = os.path.join(dirtab[dirindex], name)
                except KeyError:
                    pass

    return fulltab

def find_local(ofile, ftab, sfile, lnum):
    """Find a local declared in sfile at lnum, returning DW_AT_location.

    Requires for reference: object file path, file table from get_file_table
    """

    dump = subprocess.check_output(["arm-none-eabi-objdump", "--dwarf=info", "--", ofile]).decode("ascii")

    stmt_list = None
    location = None
    decl_file = None
    decl_line = None

    for line in dump.split("\n"):
        if "DW_AT_stmt_list" in line:
            _, _, sl = line.partition(":")
            stmt_list = sl.strip()
        elif "DW_AT_location" in line:
            _, _, loc = line.partition(":")
            if "location list" not in loc:
                continue
            location = loc.split(None, 1)[0]
        elif "DW_AT_decl_file" in line:
            _, _, df = line.partition(":")
            decl_file = df.strip()
        elif "DW_AT_decl_line" in line:
            _, _, dl = line.partition(":")
            decl_line = dl.strip()
            try:
                path = ftab[(stmt_list, decl_file)]
            except KeyError:
                continue

            if path == sfile and int(decl_line) == lnum:
                return location

def get_pc_ranges(ofile, location):
    dump = subprocess.check_output(["arm-none-eabi-objdump", "--dwarf=loc", "--", ofile],
            stderr=subprocess.PIPE).decode("ascii")
    found = False

    ranges = []

    for line in dump.split("\n"):
        try:
            index, pcrange = line.split(None, 1)
        except ValueError:
            continue
        else:
            if "<End of list>" in pcrange:
                found = False
            else:
                try:
                    if int(index, 16) == int(location, 0):
                        found = True
                except ValueError:
                    continue

            if found:
                start, end = pcrange.split(None, 2)[0:2]
                ranges.append((int(start, 16), int(end, 16)))

    return ranges

class Range(gdb.Command):
    def __init__(self):
        super(Range, self).__init__("range", gdb.COMMAND_USER)

    def invoke(self, arg, from_tty):
        s = gdb.lookup_symbol(arg)
        line = s[0].line
        sfile = s[0].symtab.filename
        ofile = s[0].symtab.objfile.filename
        arch = gdb.selected_frame().architecture().name()

        assert arch == "arm"

        ft = get_file_table(ofile)
        loc_index = find_local(ofile, ft, sfile, line)
        for start, end in get_pc_ranges(ofile, loc_index):
            print("%x -- %x" % (start, end))

Range()
