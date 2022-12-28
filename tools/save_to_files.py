#!/usr/bin/env python3
"""Receive data from Spektri by ZeroMQ and save it to files."""

import struct
import time

import zmq

import spektri

zctx = spektri.zctx

def main(
    # Subscribe to any uint8 spectrum data by default
    sub_topic = spektri.spectrum_topic(),
    address = "ipc:///tmp/spektri.zmq",
    filename_fmt = "../data/test_%Y%m%d_%H%M%S",
    file_interval = 60
):
    s = zctx.socket(zmq.SUB)

    #s.setsockopt(zmq.RCVBUF, 100000)
    #s.set_hwm(10)

    s.subscribe(sub_topic)
    s.connect(address)

    output_file = None

    prev_topic = None
    new_file = True
    prev_t_s = 0
    while True:
        topic, msg = s.recv_multipart()
        # Start a new file if topic changes
        if topic != prev_topic:
            print("Topic:", topic.hex())
            prev_topic = topic
            new_file = True

        if True:
            # Write data to a file
            #output_file.write(msg)
            # Write only the signal without the timestamps etc
            seq, t_s, t_ns = struct.unpack("<QQI", msg[0:20])
            if t_s % file_interval < prev_t_s % file_interval:
                new_file = True

            if new_file:
                if output_file != None:
                    output_file.close()
                filename = time.strftime(filename_fmt, time.gmtime(t_s))
                print("Saving in file", filename)
                output_file = open(filename, "ab")
                new_file = False

            # Bytes from 8 to 20 are compatible with the current spectrogram viewer.
            # This is a temporary hack until the file format is more stable.
            output_file.write(msg[8:20])
            # Spectrum or signal data
            output_file.write(msg[24:])

            prev_t_s = t_s


if __name__ == "__main__":
    import sys
    if len(sys.argv) == 4:
        # TODO: use argv[1] to select which data to save.
        # Now it only uses the default value.
        main(filename_fmt = sys.argv[2], file_interval = int(sys.argv[3]))
    elif len(sys.argv) == 5:
        main(filename_fmt = sys.argv[2], file_interval = int(sys.argv[3]), address = sys.argv[4])
    else:
        main()
