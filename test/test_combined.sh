#!/bin/bash
# Test spectrum analysis and filter bank with a frequency sweep.
# This script also tests the ZeroMQ interface
# and storage of measurement data.
# Spectrum results can be checked using gui.html.

. common.sh

FFTSIZE=2048
SAMPLERATE=2048000
FORMAT=f32le

# Filter parameters to test
FS0=32000
FC0=0
FS1=32000
FC1=80000
FS2=64000
FC2=100000

# zmq_rx.py may miss some of the first and last messages,
# so the results may be slightly different every time.
# This could possibly be fixed by adding some kind of synchronization
# between the programs, but it's not really needed for actual use,
# so that's not implemented. "sleep 1" is used instead.
# If processing a complete data file is critical, file output should
# be used instead of ZMQ anyway.

../tools/save_to_files.py spectrum "data/test_%Y%m%d_%H%M%S_${SAMPLERATE}_${FFTSIZE}_8_T.data" 86400 &
./zmq_rx.py "$FS0" "$FC0" "data/test_${FS0}_${FC0}.cf32" &
./zmq_rx.py "$FS1" "$FC1" "data/test_${FS1}_${FC1}.cf32" &
./zmq_rx.py "$FS2" "$FC2" "data/test_${FS2}_${FC2}.cf32" &

(
    sleep 1
    time testsignal "--format=$FORMAT" --samples=150000000
    sleep 1
) | \
(time spektri \
    "--samplerate=$SAMPLERATE" "--inputformat=$FORMAT" \
    "--fftsize=$FFTSIZE" "--averages=100" \
    "--filters" \
        "fs=${FS0}:fc=${FC0}" \
        "fs=${FS1}:fc=${FC1}" \
        "fs=${FS2}:fc=${FC2}" \
) >/dev/null

# Feed filtered signals into Spektri again
# so that the output spectrum can be checked using gui.html.
# I just realized, however, that this idea does not actually work that well
# because support for complex signals is still a bit work in progress.

FS_PLOT=$FS2
FC_PLOT=$FC2

# Use another ZeroMQ address to avoid confusion with the previous instance.
ADDR2="ipc:///tmp/spektri2"

../tools/save_to_files.py spectrum "data/filtered_%Y%m%d_%H%M%S_${FC_PLOT}_2046_8_T.data" 86400 "$ADDR2" &

(
    sleep 1
    cat "data/test_${FS_PLOT}_${FC_PLOT}.cf32"
    sleep 1
) | \
spektri \
    "--zmqbind=$ADDR2" \
    "--samplerate=$FS_PLOT" "--centerfreq=$FC_PLOT" "--inputformat=cf32le" \
    "--fftsize=1024" "--averages=1" \
    < "data/test_${FS_PLOT}_${FC_PLOT}.cf32" \
    > /dev/null
