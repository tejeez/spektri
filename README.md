This is my attempt to implement an FFT-based spectrum analyzer program
that can make use of parallel computation on multi-core CPUs.

The same FFT results will be also used for digital down-conversion
of multiple narrow-band channels using a
[fast-convolution filter bank](https://ieeexplore.ieee.org/document/6834830).

It's related to my master's thesis on implementing a direct sampling
0-30 MHz receiver within a limited power budget.
The code will be benchmarked primarily on a Raspberry Pi 4 in order to
determine whether it's feasible to perform the signal processing on a
modern single-board computer consuming a few watts of power.

Hopefully, the code will be useful for other purposes as well.
