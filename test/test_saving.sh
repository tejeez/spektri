#!/bin/bash
. common.sh

FFTSIZE=1024
SAMPLERATE=10000000

# save_to_files.py may miss some of the first and last messages,
# so the results may be slightly different every time.
# Limit the rate using pv to test splitting to files.

save_to_files.py spectrum "data/test_%Y%m%d_%H%M%S_${SAMPLERATE}_${FFTSIZE}_8_T.data" 5 & PID1=$!

testsignal real 200000000 \
| pv -L $(expr $SAMPLERATE \* 2) \
| (time spektri --inputformat=s16le  --samplerate=$SAMPLERATE --fftsize=$FFTSIZE --averages=1000) >/dev/null

kill $PID1
