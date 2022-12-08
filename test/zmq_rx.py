#!/usr/bin/env python3
"""Test receiving data from Spektri by ZeroMQ."""

# TODO: Get rid of this script and turn it into a proper tool for saving
# filter bank output. It could be a part of save_to_files.py too.

import sys
import struct

import zmq

# Use some functions from ../tools/spektri.py
sys.path.insert(0, "../tools")
import spektri

zctx = spektri.zctx

def main(fs = 32000, fc = 0, filename = "data/test", address = "ipc:///tmp/spektri.zmq"):
    s = zctx.socket(zmq.SUB)

    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)

    sub_topic = spektri.signal_topic(fs, fc)
    s.subscribe(sub_topic)

    s.connect(address)

    output_file = open(filename, "wb")

    # Which sequnce number is expected next
    next_seq = 0

    while True:
        topic, msg = s.recv_multipart()
        assert topic == sub_topic

        # Test sequence numbers
        metadata = spektri.unpack_metadata(msg)
        if metadata.seq != next_seq:
            print(f"Lost messages? Expected sequence number {next_seq}, received {metadata.seq}")
        next_seq = (metadata.seq + 1) & (2**64-1)

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
