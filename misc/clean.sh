#!/bin/sh

# Functions
function clean_rust_projects() {
    for i in $@; do
        cd $i
        rm -rf target/ Cargo.lock
        cd ${OLDPWD}
    done
}

# Navigate to `/`
cd $(dirname $0)
cd ../

# Navigate to `/misc`
cd misc
clean_rust_projects toolkit music_convert
cd ../

# Navigate to `/experiments`
cd experiments
clean_rust_projects trustedcell/simpletrustedcelld trustedcell/trustedcelld  txtfmt app-sandbox fat32x wav2bmp bmp2wav fat32x
cd ../

# Navigate to `/archive`
cd archive
clean_rust_projects randvoca
cd ../
