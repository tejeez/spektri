#!/bin/sh
# HF spectrum analyzer for RX888

FFTSIZE=16384
SAMPLERATE=128000000
# Sample rate requested from the receiver. It may differ from
# the nominal sample rate above if frequency calibration is needed.
SAMPLERATE_R=127997610

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

../tools/save_to_files.py spectrum "../data/hf_%Y%m%d_%H%M%S_${SAMPLERATE}_${FFTSIZE}_8_T.data" 86400 & PID1=$!

# Put sddc_stream in loop, so that it is restarted if it fails
(while true; do
 ${TASKSET2} "${LIBSDDC}/build/src/sddc_stream" "${LIBSDDC}/firmware/SDDC_FX3.img" "${SAMPLERATE_R}"
done) \
| ${TASKSET2} pv \
| ${TASKSET} ../spektri/target/release/spektri \
"--inputformat=s16le" \
"--fftsize=${FFTSIZE}" \
"--spectrumformat=u8" \
"--averages=100000" \
"--filters" \
 "freq=-16;bins=64;topic=500000 125000  " \
 "freq=208;bins=64;topic=500000 1875000  " \
 "freq=432;bins=64;topic=500000 3625000  " \
 "freq=576;bins=32;topic=250000 4625000  " \
 "freq=880;bins=64;topic=500000 7125000  " \
 "freq=1264;bins=64;topic=500000 10125000  " \
 "freq=1776;bins=64;topic=500000 14125000  " \
 "freq=3568;bins=64;topic=500000 28125000  " \
 "freq=6400;bins=64;topic=500000 50250000  " \


kill $PID1
