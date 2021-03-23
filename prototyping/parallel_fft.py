#!/usr/bin/env python3
"""Test for splitting an FFT into 4 parts by performing the last radix-4 step
outside of the FFT library being used.

This allows parallelizing FFT into 4 threads even when the FFT library does
not support parallel computation by itself.

Note that this Python code does not actually compute it in parallel."""

import numpy as np

N = 65536 # FFT size
# Generate some random signal
s = np.random.random(N) + np.random.random(N) * 1j

# Simple FFT for comparison
f1 = np.fft.fft(s)

# Twiddle factors
twiddles = [np.exp(-1j * np.linspace(0, np.pi*2 *i/4, N//4, endpoint=False)) for i in range(4)]

# Split it into 4 parts
fp = [np.fft.fft(s[i : N : 4]) * twiddles[i] for i in range(4)]

f2 = np.zeros(N, dtype=np.complex128)

f2[N//4*0 : N//4*1] = fp[0] +      fp[1] + fp[2] +      fp[3]
f2[N//4*1 : N//4*2] = fp[0] - 1j * fp[1] - fp[2] + 1j * fp[3]
f2[N//4*2 : N//4*3] = fp[0] -      fp[1] + fp[2] -      fp[3]
f2[N//4*3 : N//4*4] = fp[0] + 1j * fp[1] - fp[2] - 1j * fp[3]

def power(v):
    """Energy of a signal"""
    return sum(v.real**2) + sum(v.imag**2)

def show_comparison(a, b):
    """Show the "energy" of both results and their difference."""
    print("%E %E %E" % (power(a), power(b), power(a-b)))

show_comparison(f1, f2)

# Also show the difference separately for all 4 parts of the result.
# This helps to see if only some of them are calculated wrong.
for i in range(4):
    show_comparison(f1[N//4 * i : N//4 * (i+1)], f2[N//4 * i : N//4 * (i+1)])
