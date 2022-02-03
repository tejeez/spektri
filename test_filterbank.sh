#!/bin/bash
set -e -x
mkdir -p data/
# Compile the programs
(cd testsignal; cargo build --release)
(cd spektri; cargo build --release)
# Test the spectrum analyzer with a frequency sweep

# Common parameters
P="--fftsize=1024 --averages=100"

# Filter parameters. Test both ZMQ and file output.
F="--filters freq=100;bins=32;topic=test1 freq=200;bins=32;topic=test2 freq=-300;bins=64;file=data/test3.spektri"

# zmq_rx.py may miss some of the first and last messages,
# so the results may be slightly different every time.
prototyping/zmq_rx.py test1 data/test1.cf32 & PID1=$!
prototyping/zmq_rx.py test2 data/test2.cf32 & PID2=$!
time testsignal/target/release/testsignal complex 100000000 | (time spektri/target/release/spektri --inputformat=cs16le $P $F)
kill $PID1 $PID2
