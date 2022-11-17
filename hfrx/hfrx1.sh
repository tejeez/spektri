#!/bin/sh
# HF spectrum analyzer for RX888

FFTSIZE=65536
SAMPLERATE=131072000
# Sample rate requested from the receiver. It may differ from
# the nominal sample rate above if frequency calibration is needed.
SAMPLERATE_R=131069550

# libsddc repository should be cloned into ../../libsddc
# and built into ../../libsddc/build
LIBSDDC="../../libsddc"
DATA="../data"
mkdir -p "${DATA}"

# On a hyperthreading machine, it seems to be more efficient
# to use only one logical core from each physical core
# for the heavy computation.
# This is only used for testing on a i5-3360M laptop.
# On non-hyperthreading machines, these should be removed.
#TASKSET="taskset -c 0,2"
# Run the other processes on the other logical cores.
#TASKSET2="taskset -c 1,3"

# For other machines
TASKSET=
TASKSET2=

# Priorities
RT=" chrt -r 10"
RT2=" chrt -r 11"

../tools/save_to_files.py spectrum "../data/hf_%Y%m%d_%H%M%S_${SAMPLERATE}_${FFTSIZE}_8_T.data" 86400 & PID1=$!

# Put sddc_stream in loop, so that it is restarted if it fails
(while true; do
 ${TASKSET2} ${RT2} "${LIBSDDC}/build/src/sddc_stream" "${LIBSDDC}/firmware/SDDC_FX3.img" "${SAMPLERATE_R}"
done) \
| ${TASKSET2} ${RT2} pv \
| ${TASKSET} ${RT} ../spektri/target/release/spektri \
"--inputformat=s16le" \
"--samplerate=${SAMPLERATE}" \
"--centerfreq=0" \
"--fftsize=${FFTSIZE}" \
"--spectrumformat=u8" \
"--averages=24000" \
"--filters" \
 "fs=64000:fc=80000" \
 "fs=64000:fc=1848000" \
 "fs=64000:fc=3696000" \
 "fs=64000:fc=4624000" \
 "fs=64000:fc=7056000" \
 "fs=64000:fc=14264000" \
>/dev/null

kill $PID1
