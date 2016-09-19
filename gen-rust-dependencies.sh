#!/bin/bash

find . -name rustsrc -prune -o \( -name '*.rs' -print0 \) | while read -d $'\0' srcfile; do

    grep 'extern crate' "$srcfile" | awk '{print $3}' | sed 's/;//' | while read dep; do

        location="$(find . -name rustsrc -prune -o \( -name "${dep}.rs" -print -quit \) | head -n 1)"
        if [[ x"$location" == x"" ]]; then
            continue
        fi
        compiled="$(basename "${location}" | sed -e 's/^/lib/' -e 's/\.rs$/\.rlib/')"
        fullpath="$(dirname "${location}")/${compiled}"
        echo "$srcfile: $fullpath"

    done

done
