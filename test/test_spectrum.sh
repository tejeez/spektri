#!/bin/bash
# Test the spectrum analyzer with a frequency sweep.

. common.sh

# Common parameters
P="--samplerate=1024000 --fftsize=1024 --averages=100"

# Test with complex input, real input and a different input format.
for format in cf32le f32le s16le
do
    time testsignal "--format=$format" --samples=100000000 | (time spektri "--inputformat=$format" $P) > "data/testspectrum_${format}.data"
done

# Test another spectrum output format.
time testsignal --format=f32le --samples=100000000 | (time spektri --inputformat=s16le --spectrumformat=u16 $P) > data/testspectrum_f32le_16.data

# Display the resulting spectrogram by interpreting the output as a raw image file.
# Now that 24 bytes of metadata has been added to beginning of each
# measurement record, it appears in the left side of the image
# as some extra pixels.
display -size 1048x1302 -depth 8 GRAY:data/testspectrum_cf32le.data &
display -size 537x1302 -depth 8 GRAY:data/testspectrum_f32le.data &
display -size 537x1302 -depth 8 GRAY:data/testspectrum_s16le.data &

# Does this even work?
# 16-bit spectrum data has not been used that much.
# It may be broken at the moment. Fixing it is not a high priority for now.
# The format might also be changed anyway.
display -size 525x1302 -depth 16 -endian MSB GRAY:data/testspectrum_f32le_16.data &
