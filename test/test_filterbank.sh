#!/bin/bash
. common.sh

# Common parameters
P=" --samplerate=1024000 --fftsize=1024 --averages=100"

# Filter parameters. Test both ZMQ and file output.
F="--filters fs=32000:fc=0 fs=32000:fc=32000 fs=8000:fc=-8000:file=data/test3.spektri"

# zmq_rx.py may miss some of the first and last messages,
# so the results may be slightly different every time.
# This could possibly be fixed by adding some kind of synchronization
# between the programs, but it's not really needed for actual use,
# so that's not implemented. "sleep 1" is used instead.
# If processing a complete data file is critical, file output should
# be used instead of ZMQ anyway.

./zmq_rx.py 32000 0 data/test_32000_0.cf32 & PID1=$!
./zmq_rx.py 32000 32000 data/test_32000_32000.cf32 & PID2=$!

(sleep 1; time testsignal --format=cf32le --samples=100000000; sleep 1) \
| (time spektri --inputformat=cf32le $P $F) >/dev/null

kill $PID1 $PID2
