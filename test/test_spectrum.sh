#!/bin/bash
# Test the spectrum analyzer with a frequency sweep
#
# TODO: check if the test still gives correct results

. common.sh

# Common parameters
P="--samplerate=1024000 --fftsize=1024 --averages=100"

time testsignal complex 100000000 | (time spektri --inputformat=cs16le $P) > data/testspectrum.data

time testsignal real 100000000 | (time spektri --inputformat=s16le $P) > data/testspectrum_real.data

# Test another spectrum output format.
time testsignal real 100000000 | (time spektri --inputformat=s16le --spectrumformat=u16 $P) > data/testspectrum_real16.data

# Display the resulting spectrogram by interpreting the output as a raw image file.
# Now that 24 bytes of metadata has been added to beginning of each
# measurement record, it appears in the left side of the image
# as some extra pixels.
display -size 1048x1302 -depth 8 GRAY:data/testspectrum.data &
display -size 537x1302 -depth 8 GRAY:data/testspectrum_real.data &
display -size 525x1302 -depth 16 -endian MSB GRAY:data/testspectrum_real16.data &
