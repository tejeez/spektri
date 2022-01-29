#!/bin/bash
set -e -x
mkdir -p data/
# Compile the programs
(cd testsignal; cargo build --release)
(cd spektri; cargo build --release)
# Test the spectrum analyzer with a frequency sweep

# Common parameters
P="--fftsize=1024 --averages=100"

# Filter parameters
F="--filters freq=100;bins=32;file=/dev/null"

time testsignal/target/release/testsignal complex 1000000000 | (time spektri/target/release/spektri --inputformat=cs16le $P $F) > /dev/null
