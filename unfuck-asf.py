#!/usr/bin/env python3
# Atmel ASF include path unfucker
# Instead of having to include every single subdirectory, after running this
# script the ASF will #include <asf/dir/subdir/file.h> and stop polluting your
# project.

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

def usage():
    print("unfuck-asf PLATFORM OLDPATH NEWPATH")

def get_include_path(subdir):
    incpath = {}
    for root, dirs, files in os.walk(subdir):
        for name in files:
            p = os.path.join(root, name)
            incpath[name] = p
    return incpath

INC_RE = re.compile(r'^\s*#include\s+[<"]([^>"]+)[>"]')

def fix_one_file(fn, incpath):
    lines = HEADER_LIST[:]
    with open(fn) as f:
        for line in f:
            match = INC_RE.match(line)
            if match is None:
                lines.append(line)
                continue

            header = match.group(1)
            if header.startswith("conf_"):
                quoted = True
                fullpath = header
            else:
                quoted = False
                fullpath = incpath.get(header, header)

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

    for root, dirs, files in os.walk(newpath):
        for name in files:
            if not name.endswith(".h") and not name.endswith(".c"):
                continue
            fp = os.path.join(root, name)
            print("Fixup %s" % fp)
            fix_one_file(fp, incpath)

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
