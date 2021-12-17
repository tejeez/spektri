#!/bin/bash
set -e -x
mkdir -p data/
# Compile the programs
(cd testsignal; cargo build --release)
(cd spektri; cargo build --release)
# Test the spectrum analyzer with a frequency sweep
P="--fftsize=1024 --averages=100 --filters data/filtered"
time testsignal/target/release/testsignal complex 100000000 | (time spektri/target/release/spektri --inputformat=cs16le $P) > data/testspectrum.data
P+=_real
time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le $P) > data/testspectrum_real.data
# Test another output format
time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le --spectrumformat=u16 $P) > data/testspectrum_real16.data
# Display the resulting spectrogram by interpreting the output as a raw image file
display -size 1024x1302 -depth 8 GRAY:data/testspectrum.data &
display -size 513x1302 -depth 8 GRAY:data/testspectrum_real.data &
display -size 513x1302 -depth 16 -endian MSB GRAY:data/testspectrum_real16.data &
