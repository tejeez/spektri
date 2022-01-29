#!/usr/bin/env python3
"""Test receiving data from Spektri by ZeroMQ."""

import zmq

zctx = zmq.Context()

def main(sub_topic = "test", address = "ipc:///tmp/spektri.zmq", filename = "../data/zmq_rx_output"):
    s = zctx.socket(zmq.SUB)

    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)

    s.subscribe("m" + sub_topic)
    s.subscribe("d" + sub_topic)

    s.connect(address)

    output_file = open(filename, "wb")

    metadata = b""
    while True:
        topic, msg = s.recv_multipart()
        if topic[0:1] == b"m" and msg != metadata:
            # Print metadata if it has changed
            print("Metadata:", metadata)
            metadata = msg

        if topic[0:1] == b"d":
            # Write data to a file
            #output_file.write(msg)
            # Write only the signal without the timestamps etc
            output_file.write(msg[24:])

if __name__ == "__main__":
    main()
