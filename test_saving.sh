#!/bin/bash
set -e -x
mkdir -p data/
# Compile the programs
(cd testsignal; cargo build --release)
(cd spektri; cargo build --release)

FFTSIZE=1024
SAMPLERATE=10000000

# save_to_files.py may miss some of the first and last messages,
# so the results may be slightly different every time.
# Limit the rate using pv to test splitting to files.

tools/save_to_files.py spectrum "data/test_%Y%m%d_%H%M%S_${SAMPLERATE}_${FFTSIZE}_8_T.data" 5 & PID1=$!

testsignal/target/release/testsignal real 200000000 \
| pv -L $(expr $SAMPLERATE \* 2) \
| (time spektri/target/release/spektri --inputformat=s16le --fftsize=$FFTSIZE --averages=1000)

kill $PID1
