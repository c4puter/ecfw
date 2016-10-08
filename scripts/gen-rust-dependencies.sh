#!/bin/bash

find_header() {
    find . -name resources -prune -o \( -name "$1" -print \) |
        awk '{n=gsub("/","/",$0); printf "%04d/%s\n",n,$0}' |
        sort -t/ |
        sed 's|[^/]*/||'
}

find . -name resources -prune -o \( -name '*.rs' -print0 \) | while read -d $'\0' srcfile; do

    compiledsrc_base="$(basename "${srcfile}" | sed -e 's/^/lib/' -e 's/\.rs$/\.rlib/')"
    compiledsrc="$(dirname "${srcfile}")/${compiledsrc_base}"

    echo -n "$compiledsrc:"
    grep 'extern crate' "$srcfile" | awk '{print $3}' | sed 's/;//' | while read dep; do

        without_bindgen="$(echo "$dep" | sed -e 's/^bindgen_//')"
        location_rust="$(find . -name resources -prune -o \( -name "${dep}.rs" -print -quit \) | head -n 1)"
        location_c="$(find_header "${without_bindgen}.h" | head -n 1)"

        if [[ "$location_rust" != "" ]]; then
            location="$location_rust"
        elif [[ "$location_c" != "" && "$dep" == bindgen* ]]; then
            echo -n " rustsys/ctypes.rs"
            rustfn="$(basename "${location_c}" | sed -e 's/\.h$/\.rs/' -e 's/^/bindgen_/')"
            location="$(dirname "${location_c}")/$rustfn"
        else
            continue
        fi
        compiled="$(basename "${location}" | sed -e 's/^/lib/' -e 's/\.rs$/\.rlib/')"
        fullpath="$(dirname "${location}")/${compiled}"
        echo -n " $location $fullpath"

    done

    echo

done
