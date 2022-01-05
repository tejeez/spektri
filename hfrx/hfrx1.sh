#!/bin/sh
# HF spectrum analyzer for RX888

FFTSIZE=16384
SAMPLERATE=128000000

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

# Put sddc_stream in loop, so that it is restarted if it fails
(while true; do
 ${TASKSET2} "${LIBSDDC}/build/src/sddc_stream" "${LIBSDDC}/firmware/SDDC_FX3.img" "${SAMPLERATE}"
done) \
| ${TASKSET2} pv \
| ${TASKSET} ../spektri/target/release/spektri \
"--inputformat=s16le" \
"--fftsize=${FFTSIZE}" \
"--spectrumformat=u8" \
"--averages=20000" \
> "${DATA}/hf_$(date +%Y%m%d_%H%M%S)_${SAMPLERATE}_${FFTSIZE}_8_T.data"
