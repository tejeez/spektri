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
F="--filters freq=0;bins=32;file=data/filtered1 freq=100;bins=64;file=data/filtered2"

time testsignal/target/release/testsignal complex 100000000 | (time spektri/target/release/spektri --inputformat=cs16le $P $F) > data/testspectrum.data

# Use different output files for tests with real input signal
F="--filters freq=0;bins=32;file=data/filtered1_real freq=100;bins=64;file=data/filtered2_real"

time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le $P $F) > data/testspectrum_real.data

# Test another spectrum output format.
# Skip testing filters this time because the same filters were run already.
time testsignal/target/release/testsignal real 100000000 | (time spektri/target/release/spektri --inputformat=s16le --spectrumformat=u16 $P) > data/testspectrum_real16.data

# Display the resulting spectrogram by interpreting the output as a raw image file
display -size 1024x1302 -depth 8 GRAY:data/testspectrum.data &
display -size 513x1302 -depth 8 GRAY:data/testspectrum_real.data &
display -size 513x1302 -depth 16 -endian MSB GRAY:data/testspectrum_real16.data &
