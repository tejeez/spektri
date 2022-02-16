#!/usr/bin/env python3
"""Demodulators"""

import numpy as np
import numba as nb
from scipy import signal

import ddc


@nb.jitclass([
    ('pa', nb.float32),
])
class SsbAgc:
    """AGC for SSB signals."""
    def __init__(self):
        self.pa = 0

    def execute(self, input):
        attack = nb.float32(0.001)
        release = nb.float32(0.0002)

        pa = self.pa
        output = np.zeros_like(input)

        for i in range(len(input)):
            # Input sample
            s = input[i]
            # Power of the input sample
            p = s.real ** 2 + s.imag ** 2
            # Difference from the average power
            pd = p - pa
            if pd >= 0:
                pa += pd * attack
            else:
                pa += pd * release

            # Normalize the amplitude based on average power
            if pa > 0:
                s *= nb.float32(0.3) / np.sqrt(pa)
            else:
                # this shouldn't happen often
                s = 0

            # Some samples may still be above 1, so clip them
            p = s.real ** 2 + s.imag ** 2
            if p > 1:
                s *= nb.float32(1) / np.sqrt(p)

            output[i] = s

        self.pa = pa
        return output


class SsbDemodulator:
    """Demodulate SSB by Weaver method.

    Output is audio at a sample rate of 8 kHz."""
    def __init__(self, fs_in, fc_in, fc_demod, mode='lsb'):
        sideband = 1 if mode == 'usb' else -1
        # Design the DDC to move center frequency to 0
        # and resample the signal to 16 kHz
        self.ddc = ddc.DesignDdc(fs_in, 16000, fc_demod + 1500*sideband - fc_in)

        # Follow the resampler by one more filtering stage
        self.filts = signal.butter(10, 1300, fs=16000, output='sos')
        self.filtz = signal.sosfilt_zi(self.filts) * np.complex64(0)

        # Mixer to shift the signal from 0 center frequency to audio band.
        # Also decimate by 2. The signal has been filtered already, so
        # decimation can be done by just dropping every second sample.
        # Let's use RationalDdc here as well for simplicity,
        # even though it's not the most efficient solution.
        self.mixer2 = ddc.RationalDdc(np.ones(1, dtype=np.float32), decimation=2, freq_num=3*sideband, freq_den=32)

        self.agc = SsbAgc()

    def execute(self, input):
        s = self.ddc.execute(input)
        s, self.filtz = signal.sosfilt(self.filts, s, zi=self.filtz)
        s = self.mixer2.execute(s)
        s = self.agc.execute(s)
        return s.real


def benchmark(fs_in = 500000, buflen = 512, repeats = 1000):
    """Benchmark the polyphase DDC algorithm."""
    import time

    demod = SsbDemodulator(fs_in, 3625000, 3699000, 'lsb')

    signalin = \
        (np.random.normal(size=buflen) + \
         np.random.normal(size=buflen) *1j) \
        .astype(np.complex64)
    # Call it once to make sure numba has compiled it
    # before measuring execution time.
    signalout = demod.execute(signalin)

    t1 = time.perf_counter_ns()
    for _ in range(repeats):
        signalout = demod.execute(signalin)
    t2 = time.perf_counter_ns()
    samples_per_nanosecond = buflen * repeats / (t2-t1)
    print("%.3f MS/s" % (samples_per_nanosecond * 1e3))

if __name__ == "__main__":
    import sys
    benchmark()
