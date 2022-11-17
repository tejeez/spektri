#!/bin/sh
# Make SSB demodulated audio available through pulseaudio.
# This way, it can be fed to digital mode decoders such as WSJT-X.

export PULSE_SERVER=/run/user/$(id -u)/pulse/native

demod()
{
pactl load-module module-pipe-source source_name=demod_$3 file=/tmp/audiopipe_demod_$3 format=float32le rate=8000 channels=1
sleep 10
../tools/demodulate.py "${SPEKTRIADDR:=ipc:///tmp/spektri.zmq}" $1 $2 $3 usb > /tmp/audiopipe_demod_$3
}

# FT-8 frequencies for a few amateur radio bands
# TODO: fix filter bank output frequencies, add them to hfrx1.sh if needed
demod 500000 3625000 3573000 &
demod 500000 7125000 7074000 &
demod 500000 14125000 14074000 &
demod 500000 28125000 28074000 &
sleep infinity
