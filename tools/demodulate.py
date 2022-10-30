#!/usr/bin/env python3
"""Receive data from Spektri by ZeroMQ and demodulate it."""

import sys

from demodulator import SsbDemodulator, AmDemodulator, IqDemodulator
import spektri

def main(address='ipc:///tmp/spektri.zmq', fs_in=500000, fc_in=3625000, fc_demod=3699000, mode='lsb'):
    if mode == 'iq':
        demod = IqDemodulator(fs_in, fc_in, fc_demod, mode)
    elif mode == 'am':
        demod = AmDemodulator(fs_in, fc_in, fc_demod, mode)
    else:
        demod = SsbDemodulator(fs_in, fc_in, fc_demod, mode)

    # Pre-evaluate some methods as suggested at
    # https://wiki.python.org/moin/PythonSpeed/PerformanceTips#Avoiding_dots...
    # It did not really speed it up a lot though.
    demod_execute = demod.execute
    write_output = sys.stdout.buffer.write

    for _, signal in spektri.recv_signal(fs_in, fc_in, address=address):
        d = demod_execute(signal)
        write_output(d.tobytes())


if __name__ == "__main__":
    argv = sys.argv
    if len(argv) == 6:
        main(address = argv[1], fs_in = int(argv[2]), fc_in = int(argv[3]), fc_demod = int(argv[4]), mode = argv[5])
    else:
        main()
