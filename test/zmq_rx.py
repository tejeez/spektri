#!/usr/bin/env python3
"""Test receiving data from Spektri by ZeroMQ."""

import struct

import zmq

zctx = zmq.Context()

def serialize_signal_topic(fs, fc):
    return bytes((2, 0x50, 0,0,0,0,0,0)) + struct.pack("<dd", fs, fc)

def main(fs = 32000, fc = 0, filename = "data/test", address = "ipc:///tmp/spektri.zmq"):
    s = zctx.socket(zmq.SUB)

    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)

    sub_topic = serialize_signal_topic(fs, fc)
    s.subscribe(sub_topic)

    s.connect(address)

    output_file = open(filename, "wb")

    while True:
        topic, msg = s.recv_multipart()
        assert topic == sub_topic
        # Write data to a file
        #output_file.write(msg)
        # Write only the signal without the timestamps etc
        output_file.write(msg[24:])

if __name__ == "__main__":
    import sys
    if len(sys.argv) == 4:
        main(fs = float(sys.argv[1]), fc = float(sys.argv[2]), filename = sys.argv[3])
    else:
        main()
