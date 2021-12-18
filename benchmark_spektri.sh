#!/bin/bash
set -e -x
cd spektri
cargo build --release
FILTER="freq=1000;bins=32;file=/dev/null"
PARAMS="--fftsize=16384 --filters $FILTER $FILTER $FILTER"
dd bs=8192 count=200k status=progress if=/dev/zero | (time target/release/spektri --inputformat=cs16le $PARAMS >/dev/null)
dd bs=8192 count=200k status=progress if=/dev/zero | (time target/release/spektri --inputformat=s16le $PARAMS >/dev/null)
