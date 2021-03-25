#!/bin/bash
set -e -x
# Compile the programs
(cd testsignal; cargo build --release)
(cd spektri; cargo build --release)
# Test the spectrum analyzer with a frequency sweep
time testsignal/target/release/testsignal complex 100000000 | (time spektri/target/release/spektri complex 1024) > data/testspectrum.data
time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri real 1024) > data/testspectrum_real.data
# Display the resulting spectrogram by interpreting the output as a raw image file
display -size 1024x1302 -depth 8 GRAY:data/testspectrum.data &
display -size 513x1302 -depth 8 GRAY:data/testspectrum_real.data &
