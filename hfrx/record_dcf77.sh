#!/bin/sh
# Record the DCF77 signal, useful for frequency calibration

set -e -x
DEMODULATE='../tools/demodulate.py ipc:///tmp/spektri.zmq'
${DEMODULATE} 500000 125000 77500 iq >> ../data/dcf77_$(date +%s).cf32
