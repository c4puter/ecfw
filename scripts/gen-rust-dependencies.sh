#!/bin/bash

EXCLUDES="-name rustsrc -o -name asf -o -name asf-unf"

find_header() {
    find . \( $EXCLUDES -prune \) -o \( -name "$1" -print \) |
        awk '{n=gsub("/","/",$0); printf "%04d/%s\n",n,$0}' |
        sort -t/ |
        sed 's|[^/]*/||'
}

find . -name rustsrc -prune -o \( -name '*.rs' -print0 \) | while read -d $'\0' srcfile; do

    grep 'extern crate' "$srcfile" | awk '{print $3}' | sed 's/;//' | while read dep; do

        without_bindgen="$(echo "$dep" | sed -e 's/^bindgen_//')"
        location_rust="$(find . \( $EXCLUDES -prune \) -o \( -name "${dep}.rs" -print -quit \) | head -n 1)"
        location_c="$(find_header "${without_bindgen}.h" | head -n 1)"

        if [[ x"$location_rust" != x"" ]]; then
            location="$location_rust"
        elif [[ x"$location_c" != x"" ]]; then
            echo "$srcfile: rustsys/libctypes.rlib"
            rustfn="$(basename "${location_c}" | sed -e 's/\.h$/\.rs/' -e 's/^/bindgen_/')"
            location="$(dirname "${location_c}")/$rustfn"
        else
            continue
        fi
        compiled="$(basename "${location}" | sed -e 's/^/lib/' -e 's/\.rs$/\.rlib/')"
        fullpath="$(dirname "${location}")/${compiled}"
        echo "$srcfile: $fullpath"

    done

done
