#!/usr/bin/env python3
"""Generate random numbers to use for testing
the Rust implementation of two real FFTs."""

import numpy as np

N = 8 # FFT size
# Generate two real-valued random signals and a window function
win = np.random.random(N)
s0 = np.random.random(N)
s1 = np.random.random(N)

# Simple FFTs for comparison.
# Save only the bins from 0 to N/2, since the input is real-valued.
fs0 = np.fft.fft(s0 * win) [0 : (N//2+1)]
fs1 = np.fft.fft(s1 * win) [0 : (N//2+1)]

# Print them in a format that can be copy-pasted to the test function
for a in (win, s0, s1):
    print(','.join(('%f' % v) for v in a))

for a in (fs0, fs1):
    print(','.join(('Complex{re:%f,im:%f}' % (np.real(v), np.imag(v))) for v in a))
