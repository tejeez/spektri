#!/usr/bin/env python3
"""Receive data from Spektri by ZeroMQ and demodulate it."""

import struct
import sys

import numpy as np
import zmq

from demodulator import SsbDemodulator

zctx = zmq.Context()


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
