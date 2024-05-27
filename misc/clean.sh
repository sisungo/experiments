#!/bin/sh

# Functions
function clean_rust_projects() {
    for i in $@; do
        cd $i
        rm -rf target/ Cargo.lock
        cd ../
    done
}

# Navigate to `/`
cd $(dirname $0)
cd ../

# Navigate to `/misc`
cd misc
clean_rust_projects toolkit music_convert app_2201_novels web_proxy loginloop
cd ../

# Navigate to `/experiments`
cd experiments
clean_rust_projects txtfmt app-sandbox fat32x
cd ../

# Navigate to `/archive`
cd archive
clean_rust_projects randvoca
cd ../
