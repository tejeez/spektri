#!/bin/bash
# Test the spectrum analyzer with a frequency sweep
# TODO: Fix this test. It does not work anymore.

set -e -x
mkdir -p data/
# Compile the programs
(cd testsignal; cargo build --release)
(cd spektri; cargo build --release)

# Common parameters
P="--samplerate=1024000 --fftsize=1024 --averages=100"

time testsignal/target/release/testsignal complex 100000000 | (time spektri/target/release/spektri --inputformat=cs16le $P) > data/testspectrum.data

time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le $P) > data/testspectrum_real.data

# Test another spectrum output format.
time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le --spectrumformat=u16 $P) > data/testspectrum_real16.data

# Display the resulting spectrogram by interpreting the output as a raw image file
display -size 1024x1302 -depth 8 GRAY:data/testspectrum.data &
display -size 513x1302 -depth 8 GRAY:data/testspectrum_real.data &
display -size 513x1302 -depth 16 -endian MSB GRAY:data/testspectrum_real16.data &
