#!/bin/bash
. common.sh
FILTER="fs=80000:fc=0"
PARAMS="--fftsize=16384 --samplerate=163840000 --filters $FILTER $FILTER $FILTER"
dd bs=8192 count=200k status=progress if=/dev/zero | (time spektri --inputformat=cs16le $PARAMS >/dev/null)
dd bs=8192 count=200k status=progress if=/dev/zero | (time spektri --inputformat=s16le $PARAMS >/dev/null)
