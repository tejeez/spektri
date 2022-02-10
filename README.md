# Spektri

Spektri is an FFT-based spectrum analyzer program
that can make use of parallel computation on multi-core CPUs.

The same FFT results are also used for digital down-conversion
of multiple narrow-band channels using a
[fast-convolution filter bank](https://ieeexplore.ieee.org/document/6834830).

At the moment, most features are undocumented and unfinished.
Everything is still under development and anything may change at any point.

## Interfaces

Spektri reads its input signal from stdin, so it can be used with any
receiver hardware that provides a program to stream samples to a pipe.
So far, it has been mostly tested with an RX888 HF receiver.

Spectrum measurements and outputs of the filter bank are sent to
a ZeroMQ PUB socket, so that they can be read by other programs for further
processing of the signals.
Alternatively, the results can be saved into files, which may be a better
option for batch processing of recorded signals.

The file format and format of the ZeroMQ messages are not really stable yet.
ZeroMQ may not even be the best choice for the interface, and it was mainly
chosen because I found it easy to use and was already familiar with it.

## Contents of the repository

* [spektri/](spektri/): The spectrum analysis and filter bank program.
* [gui/](gui/): Visualization tool for spectrum files.
* [tools/](tools/): Various programs to make use of the data from Spektri.
* [hfrx/](hfrx/): Demonstration of wideband HF reception using RX888.
* [testsignal/](testsignal/): Generates a frequency sweep for testing.
* [prototyping/](prototyping/): Scripts used to test some algorithms before
  implementing them as a part of Spektri.
