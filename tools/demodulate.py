#!/usr/bin/env python3
"""Receive data from Spektri by ZeroMQ and demodulate it."""

import struct
import sys

import numpy as np
import zmq

from demodulator import SsbDemodulator, IqDemodulator

zctx = zmq.Context()


def main(address='ipc:///tmp/spektri.zmq', fs_in=500000, fc_in=3625000, fc_demod=3699000, mode='lsb'):
    # By current convention, subscription topic has the sample rate and center frequency as a string.
    sub_topic = "%d %d  " % (fs_in, fc_in)

    s = zctx.socket(zmq.SUB)

    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)

    s.subscribe("d" + sub_topic)

    s.connect(address)

    if mode == 'iq':
        demod = IqDemodulator(fs_in, fc_in, fc_demod, mode)
    else:
        demod = SsbDemodulator(fs_in, fc_in, fc_demod, mode)

    # Pre-evaluate some methods as suggested at
    # https://wiki.python.org/moin/PythonSpeed/PerformanceTips#Avoiding_dots...
    # It did not really speed it up a lot though.
    frombuf = np.frombuffer
    demod_execute = demod.execute
    write_output = sys.stdout.buffer.write

    while True:
        topic, msg = s.recv_multipart()

        if topic[0:1] == b"d":
            #seq, t_s, t_ns = struct.unpack("<QQI", msg[0:20])
            signal = frombuf(msg[24:], dtype=np.complex64)
            d = demod_execute(signal)
            write_output(d.tobytes())


if __name__ == "__main__":
    argv = sys.argv
    if len(argv) == 6:
        main(address = argv[1], fs_in = int(argv[2]), fc_in = int(argv[3]), fc_demod = int(argv[4]), mode = argv[5])
    else:
        main()
