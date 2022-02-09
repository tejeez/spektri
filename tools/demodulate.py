#!/usr/bin/env python3
"""Receive data from Spektri by ZeroMQ and demodulate it."""

import struct
import time
import sys

import numpy as np
from scipy import signal
import zmq

import ddc

zctx = zmq.Context()


class SsbDemodulator:
    """Demodulate SSB by Weaver method."""
    def __init__(self, fs_in, fc_in, fc_demod, mode='lsb'):
        sideband = 1 if mode == 'usb' else -1
        # Design the DDC to move center frequency to 0
        # and resample the signal to 16 kHz
        self.ddc = ddc.DesignDdc(fs_in, 16000, fc_demod + 1500*sideband - fc_in)

        # Follow the resampler by one more filtering stage
        self.filtb, self.filta = signal.butter(4, 1400, fs=16000)
        self.filtz = signal.lfiltic(self.filtb, self.filta, np.zeros(1))

        # Mixer to shift the signal from 0 center frequency to audio band.
        # Let's use FractionalDdc here as well for simplicity,
        # even though we're not using the resampling feature.
        self.mixer2 = ddc.RationalDdc(np.ones(1, dtype=np.float32), freq_num=3*sideband, freq_den=32)

    def execute(self, input):
        d = self.ddc.execute(input)
        f, self.filtz = signal.lfilter(self.filtb, self.filta, d, zi=self.filtz)
        m = self.mixer2.execute(f)
        return m


def main(sub_topic = "500000 3625000  ", address = "ipc:///tmp/spektri.zmq"):
    s = zctx.socket(zmq.SUB)

    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)

    s.subscribe("d" + sub_topic)

    s.connect(address)

    demod = SsbDemodulator(500000, 3625000, 3699000, 'lsb')
    #demod = SsbDemodulator(250000, 4625000, 4625000, 'usb')

    while True:
        topic, msg = s.recv_multipart()

        if topic[0:1] == b"d":
            #seq, t_s, t_ns = struct.unpack("<QQI", msg[0:20])
            signal = np.frombuffer(msg[24:], dtype=np.complex64)
            d = demod.execute(signal)
            sys.stdout.buffer.write(d.tobytes())


if __name__ == "__main__":
    #import sys
    if len(sys.argv) == 2:
        main(address = sys.argv[1])
    else:
        main()
