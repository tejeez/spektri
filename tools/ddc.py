#!/usr/bin/env python3
"""
Digital downconversion.
Can be used as a second mixing, filtering, resampling stage
after the fast convolution filter bank.
"""

from fractions import Fraction

import numpy as np
import numba as nb
from scipy import signal


@nb.jit
def _sinewave(num, den):
    """Generate a complex sine wave of frequency sample_rate*num/den.
    Length is chosen such that a continuous sine wave
    can be made by repeating the returned signal."""

    # "% den" is not absolutely necessary here, but wrapping the phase
    # using integers may avoid loss of floating point precision.
    phase = \
        (np.arange(0, num*den, num, dtype = np.int64) % den) \
        .astype(np.float32) * nb.float32(2.0 * np.pi / den)
    return np.cos(phase) + np.sin(phase) * nb.complex64(1j)


@nb.jitclass([
    # float32 would be enough for the taps, but np.dot doesn't support taking product of a real and a complex vector.
    # TODO: maybe store real and imag parts of firbuf separately as float32
    ('taps', nb.complex64[:]),
    ('interpolation', nb.uint32),
    ('decimation', nb.uint32),
    ('firlen', nb.uint64),

    ('firbuf', nb.complex64[:]),
    ('fir_i', nb.uint64),
    ('fir_phase', nb.uint64),

    ('lo_table', nb.complex64[:]),
    ('lo_phase', nb.uint64),
])
class RationalDdc:
    def __init__(self, taps, interpolation=1, decimation=1, freq_num=0, freq_den=1):
        """
        Perform frequency mixing and resampling.

        Both the resampling ratio and local oscillator frequency
        are given as exact rational numbers.

        taps: FIR coefficients designed for a sample rate of
              input sample rate times the interpolation factor.
              Length shall be a multiple of the interpolation factor.

        interpolation: Interpolation factor or the resampler

        decimation: Decimation factor of the resampler

        freq_num: Nominator of local oscillator frequency.

        freq_den: Denominator of local oscillator frequency.
        """

        # FIR parameters
        self.taps = taps.astype(np.complex64)
        self.interpolation = interpolation
        self.decimation = decimation

        # Number of coefficients in a single branch of the polyphase filter
        self.firlen = len(taps) // interpolation

        # Buffer of stored samples for FIR filtering.
        # Length is twice the FIR length for "fake circular buffering"
        self.firbuf = np.zeros(self.firlen * 2, dtype = np.complex64)
        # Index counter for firbuf
        self.fir_i = 0
        # Phase of the polyphase filter
        self.fir_phase = 0

        # Precalculate a repeating waveform for the local oscillator
        self.lo_table = _sinewave(freq_num, freq_den)
        # Index counter for the local oscillator table
        self.lo_phase = 0

    def execute(self, input):
        #output = np.zeros_like(input)
        output = np.zeros(10000, dtype=np.complex64) # TODO: size of the array
        outn = 0 # number of output samples produced
        for i in range(len(input)):
            # Mix input with local oscillator
            mixed = input[i] * self.lo_table[self.lo_phase]
            self.lo_phase = (self.lo_phase + 1) % len(self.lo_table)

            # Store to buffer.
            # Fake circular buffering by storing the samples twice.
            self.firbuf[self.fir_i] = self.firbuf[self.fir_i + self.firlen] = mixed
            self.fir_i = (self.fir_i + 1) % self.firlen

            self.fir_phase = self.fir_phase + self.interpolation
            while self.fir_phase >= self.decimation:
                self.fir_phase -= self.decimation
                assert self.fir_phase >= 0
                assert self.fir_phase < self.interpolation
                output[outn] = np.dot(
                    self.firbuf[self.fir_i : self.fir_i + self.firlen],
                    self.taps[self.fir_phase : : self.interpolation]
                )
                outn += 1
                assert outn <= len(output)
        return output[0:outn]


def DesignDdc(fs_in, fs_out, fc = 0):
    """Design a DDC for a given input sample rate, output sample rate and center frequency."""
    fs_in  = Fraction(fs_in)
    fs_out = Fraction(fs_out)
    fc     = Fraction(fc)

    freq  = -fc / fs_in

    resampling = fs_out / fs_in
    interp, decim = resampling.numerator, resampling.denominator

    taps = signal.firwin(
        (11 * decim) // interp * interp,
        1.0 / decim,
        window='blackman') \
        * interp

    return RationalDdc(
        taps,
        interpolation = interp,
        decimation = decim,
        freq_num = freq.numerator,
        freq_den = freq.denominator,
    )


def test_lo():
    """Test the local oscillator in RationalDdc.
    No resampling or filtering is done here."""
    ddc = RationalDdc(np.ones(1, dtype=np.float32), freq_num=3, freq_den=7)
    s = ddc.execute(np.ones(10, dtype=np.complex64))
    print(s)


def test(fs_in = 1000, fs_out = 300, fc = 150):
    """Test the DDC algorithm.

    Read signal from stdin, mix and resample it and write the result to stdout.
    Test by running:
    ../testsignal/target/release/testsignal complex 1000000 | ./ddc.py > ../data/ddc_test

    and check the result using, for example, Audacity.
    """
    import sys

    ddc = DesignDdc(fs_in, fs_out, fc)

    while True:
        r = sys.stdin.buffer.read(4096)
        if len(r) == 0: break
        signalin = np.frombuffer(r, dtype='int16').astype('float32').view('complex64') * (2.0**-15)
        signalout = ddc.execute(signalin)
        sys.stdout.buffer.write(signalout.tobytes())

if __name__ == "__main__":
    test()
