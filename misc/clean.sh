#!/bin/sh

# Navigate to `/`
cd $(dirname $0)
cd ../

# Navigate to `/misc`
cd misc
rm music_convert
cd ../

# Navigate to `/experiments`
cd experiments

cd randvoca
rm -r target/ Cargo.lock
cd ../

cd txtfmt
rm -r target/ Cargo.lock
cd ../

cd ../
