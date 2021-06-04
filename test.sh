#!/bin/bash
set -e -x
mkdir -p data/
# Compile the programs
(cd testsignal; cargo build --release)
(cd spektri; cargo build --release)
# Test the spectrum analyzer with a frequency sweep
time testsignal/target/release/testsignal complex 100000000 | (time spektri/target/release/spektri --inputformat=cs16le --fftsize=1024 --averages=100) > data/testspectrum.data
time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le --fftsize=1024 --averages=100) > data/testspectrum_real.data
# Test another output format
time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le --fftsize=1024 --averages=100 --spectrumformat=u16) > data/testspectrum_real16.data
# Display the resulting spectrogram by interpreting the output as a raw image file
display -size 1024x1302 -depth 8 GRAY:data/testspectrum.data &
display -size 513x1302 -depth 8 GRAY:data/testspectrum_real.data &
display -size 513x1302 -depth 16 -endian MSB GRAY:data/testspectrum_real16.data &
