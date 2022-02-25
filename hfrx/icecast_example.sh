#!/bin/sh
# Example to send demodulated audio streams to an Icecast server.

# Edit to add the password, address and port of your Icecast server.
# Modify and add frequencies as needed.

set -e -x
DEMODULATE='../tools/demodulate.py ipc:///tmp/spektri.zmq'
STREAM='ffmpeg -f f32le -ar 8000 -ac 1 -i - -acodec libopus -ab 24000 -content_type application/ogg icecast://source:password@address:port/'

${DEMODULATE} 500000 125000 76000 usb | ${STREAM}76usb.opus &
${DEMODULATE} 500000 1875000 1846000 lsb | ${STREAM}1846lsb.opus &
${DEMODULATE} 500000 3625000 3699000 lsb | ${STREAM}3699lsb.opus &
${DEMODULATE} 500000 14125000 14267000 usb | ${STREAM}14267usb.opus &
sleep infinity
