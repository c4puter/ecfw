#!/usr/bin/env python3
# Atmel ASF include path unfucker
# Instead of having to include every single subdirectory, after running this
# script the ASF will #include <asf/dir/subdir/file.h> and stop polluting your
# project.

# ASF unfucker
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

import re
import sys
import os
import subprocess

HEADER = """\
/*****************************************************************************
 * This ASF source file has been edited by unfuck-asf to have a sane include *
 * path. The edits are restricted to only #include lines, and are highly     *
 * automated, so no copyright is claimed on the modifications.               *
 *****************************************************************************/

"""

HEADER_LIST = [i + "\n" for i in HEADER.split("\n")]

INC_RE = re.compile(r'^\s*#\s*include\s+[<"]([^>"]+)[>"]')

def usage():
    print("unfuck-asf PLATFORM OLDPATH NEWPATH")

INC_IGNORES = [
    "samg", "same70", "sam3u", "sam4l", "sam3x", "samv71", "xmega",
    "uc3l", "sam4e", "sams70", "uc3a3_a4", "uc3b0_b1", "sam4cp",
    "uc3a0_a1", "sam3s", "uc3c", "samv70", "uc3d", "sam4s",
    "sam4cm", "sam3n", "sam4c", "sam4n", "mega", "sam3s8",
    "sam4cm32", "sam0", "unit_tests", "uc3"
    ]

def get_include_path(subdir):
    incpath = {}
    for root, dirs, files in os.walk(subdir):
        # Don't include device-specific files in the include path. These are
        # included relatively by headers that select the right one.
        ignored_include = False
        for i in root.split("/"):
            if i in INC_IGNORES:
                ignored_include = True
        if ignored_include:
            continue
        for name in files:
            p = os.path.join(root, name)
            incpath[name] = p
    return incpath

def try_local_resolve(fn, header, root):
    starting_dir = os.path.dirname(fn)
    assert starting_dir.startswith(root)
    starting_dir_parts = starting_dir.split('/')

    for i in range(1, len(starting_dir_parts) + 1):
        partial_parts = starting_dir_parts[0:i]
        partial = os.path.join('/'.join(partial_parts), header)
        if os.path.exists(partial):
            #print("Partial resolve %s as %s" % (header, partial))
            return partial

    return None

def fix_one_file(fn, incpath, root):
    lines = HEADER_LIST[:]
    with open(fn) as f:
        for line in f:

            # Hack to put this here, but Atmel broke this
            if "define OPTIMIZE_HIGH __attribute__" in line:
                line = line.replace("optimize(s)", "optimize(3)")

            match = INC_RE.match(line)
            if match is None:
                lines.append(line)
                continue

            header = match.group(1)
            localres = try_local_resolve(fn, header, root)
            if header == "asf.h":
                continue
            elif header.startswith("conf_"):
                quoted = True
                fullpath = header
            elif localres is not None:
                quoted = False
                fullpath = localres
            else:
                quoted = False
                header_short = os.path.split(header)[-1]
                fullpath = incpath.get(header_short, header)

            if quoted:
                lines.append('#include "%s"\n' % fullpath)
            else:
                lines.append('#include <%s>\n' % fullpath)

    with open(fn, 'w') as f:
        for i in lines:
            f.write(i)

def main(argv):
    if len(argv) != 4:
        usage()
        return 1

    platform = argv[1]
    oldpath = argv[2]
    newpath = argv[3]

    if os.path.exists(os.path.join(newpath, "drivers")):
        return 0

    if platform not in ("avr32", "mega", "xmega", "sam", "sam0"):
        print("Unknown platform: %r" % platform, file=sys.stderr)
        return 1

    if not os.path.isdir(newpath):
        os.mkdir(newpath)

    for subdir in ("common", platform):
        print("Copy %s without extra files" % subdir)

        tar_in = subprocess.Popen(
            ["tar", "-c", "--exclude=*/example", "--exclude=*_demo",
                "--exclude=*/iar", "--exclude=*/doxygen", '.'],
            cwd=os.path.join(oldpath, subdir),
            stdout=subprocess.PIPE)

        tar_out = subprocess.Popen(
            ["tar", "-x"],
            cwd=newpath,
            stdin=tar_in.stdout)

        out, err = tar_out.communicate()

    print("Calculate include path")
    incpath = get_include_path(newpath)

    print("Fixup files")
    for root, dirs, files in os.walk(newpath):
        for name in files:
            if not name.endswith(".h") and not name.endswith(".c"):
                continue
            fp = os.path.join(root, name)
            fix_one_file(fp, incpath, newpath)

    # Copy other things directly
    direct_copy = ["thirdparty/CMSIS", "thirdparty/freertos"]
    for path in direct_copy:
        os.makedirs(os.path.join(newpath, path), exist_ok=True)
        tar_in = subprocess.Popen(
                ["tar", "-c", "."],
                cwd = os.path.join(oldpath, path),
                stdout=subprocess.PIPE)
        tar_out = subprocess.Popen(
                ["tar", "x"],
                cwd = os.path.join(newpath, path),
                stdin=tar_in.stdout)
        tar_out.communicate()

if __name__ == "__main__":
    sys.exit(main(sys.argv))
