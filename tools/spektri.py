#!/usr/bin/env python3
"""Python module to receive data from Spektri."""

import struct
from dataclasses import dataclass

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


@dataclass
class Metadata:
    """Metadata of a measurement record."""
    seq: int      # Sequence number
    time_s: int   # System time, integer part in seconds
    time_ns: int  # System time, fractional part in nanoseconds

def unpack_metadata(msg):
    """Deserialize metadata of a measurement record."""
    seq, time_s, time_ns = struct.unpack("<QQI", msg[0:20])
    return Metadata(seq=seq, time_s=time_s, time_ns=time_ns)


def recv_signal(fs, fc, address=DEFAULT_ADDRESS, zctx=zctx):
    """Receive waveform data from Spektri."""

    s = zctx.socket(zmq.SUB)
    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)
    s.subscribe(signal_topic(fs, fc))
    s.connect(address)
    while True:
        _, msg = s.recv_multipart()
        yield (unpack_metadata(msg), np.frombuffer(msg[24:], dtype=np.complex64))


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
        yield (unpack_metadata(msg), np.frombuffer(msg[24:], dtype=np.complex64))
