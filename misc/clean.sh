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
clean_rust_projects trustedcell/simpletrustedcelld trustedcell/trustedcelld landlockwrap
cd ../

# Navigate to `/archive`
cd archive
clean_rust_projects randvoca fat32x txtfmt wav2bmp bmp2wav app-sandbox
cd ../
