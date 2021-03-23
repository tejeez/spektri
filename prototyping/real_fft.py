#!/usr/bin/env python3
"""Two real-to-complex FFTs calculated in one complex FFT"""

import numpy as np

N = 65536 # FFT size
# Generate two read-valued random signals
s0 = np.random.random(N)
s1 = np.random.random(N)

# Simple FFTs for comparison.
# Save only the bins from 0 to N/2, since the input is real-valued.
fs0 = np.fft.fft(s0) [0 : (N//2+1)]
fs1 = np.fft.fft(s1) [0 : (N//2+1)]

# Complex FFT, one input in real and other in imaginary part
fc = np.fft.fft(s0 + 1j * s1)

# Mirror around frequency 0 to make the other half easier to handle
fm = np.concatenate((np.array((fc[0],)), np.flip(fc[1:])))

f0 = (fc[0 : (N//2+1)] + np.conj(fm[0 : (N//2+1)])) * 0.5
f1 = (-1j*fc[0 : (N//2+1)] + np.conj(-1j*fm[0 : (N//2+1)])) * 0.5

def power(v):
    """Energy of a signal"""
    return sum(v.real**2) + sum(v.imag**2)

def show_comparison(a, b):
    """Show the "energy" of both results and their difference."""
    print("%E %E %E" % (power(a), power(b), power(a-b)))

# Compare the results
show_comparison(fs0, f0)
show_comparison(fs1, f1)
