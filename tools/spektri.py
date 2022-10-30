#!/usr/bin/env python3
"""Python module to receive data from Spektri."""

import struct

import numpy as np
import zmq

DEFAULT_ADDRESS = "ipc:///tmp/spektri.zmq"

zctx = zmq.Context()


def signal_topic(fs, fc):
    """Serialize subscription topic for waveform data
    with given sample rate and center frequency.

    Sample format is fixed as complex 32-bit float since that is
    the only output format currently supported in Spektri."""
    return bytes((2, 0x40, 0x5C, 0,0,0,0,0)) + struct.pack("<dd", fs, fc)


def spectrum_topic():
    """Serialize subscription topic for any spectrum data.

    Sample format is fixed as unsigned 8-bit int since that is
    the only output format currently supported in Spektri."""
    return bytes((2, 0x60, 0x24, 0,0,0,0,0))


def recv_signal(fs, fc, address=DEFAULT_ADDRESS, zctx=zctx):
    """Receive waveform data from Spektri."""

    s = zctx.socket(zmq.SUB)
    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)
    s.subscribe(signal_topic(fs, fc))
    s.connect(address)
    while True:
        _, msg = s.recv_multipart()
        # TODO: return metadata too. None is placeholder for that now
        yield (None, np.frombuffer(msg[24:], dtype=np.complex64))


def recv_spectrum(fs, fc, address=DEFAULT_ADDRESS, zctx=zctx):
    """Receive spectrum data from Spektri."""

    s = zctx.socket(zmq.SUB)
    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)
    s.subscribe(spectrum_topic())
    s.connect(address)
    while True:
        _, msg = s.recv_multipart()
        # TODO: return metadata too. None is placeholder for that now
        yield (None, np.frombuffer(msg[24:], dtype=np.complex64))
