#!/bin/bash
set -e -x
cd spektri
cargo build --release
dd bs=8192 count=200k status=progress if=/dev/zero | (time target/release/spektri --inputformat=cs16le --fftsize=16384 >/dev/null)
dd bs=8192 count=200k status=progress if=/dev/zero | (time target/release/spektri --inputformat=s16le --fftsize=16384 >/dev/null)
