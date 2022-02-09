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
        release = nb.float32(0.00005)

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
    """Demodulate SSB by Weaver method."""
    def __init__(self, fs_in, fc_in, fc_demod, mode='lsb'):
        sideband = 1 if mode == 'usb' else -1
        # Design the DDC to move center frequency to 0
        # and resample the signal to 16 kHz
        self.ddc = ddc.DesignDdc(fs_in, 16000, fc_demod + 1500*sideband - fc_in)

        # Follow the resampler by one more filtering stage
        self.filtb, self.filta = signal.butter(4, 1400, fs=16000)
        self.filtz = signal.lfiltic(self.filtb, self.filta, np.zeros(1))

        # Mixer to shift the signal from 0 center frequency to audio band.
        # Let's use FractionalDdc here as well for simplicity,
        # even though we're not using the resampling feature.
        self.mixer2 = ddc.RationalDdc(np.ones(1, dtype=np.float32), freq_num=3*sideband, freq_den=32)

        self.agc = SsbAgc()

    def execute(self, input):
        s = self.ddc.execute(input)
        s, self.filtz = signal.lfilter(self.filtb, self.filta, s, zi=self.filtz)
        s = self.mixer2.execute(s)
        s = self.agc.execute(s)
        return s.real

