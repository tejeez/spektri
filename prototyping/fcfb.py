#!/usr/bin/env python3
"""Implementation of a fast-convolution filter bank for testing purposes.

The implementation supports both overlap-add and overlap-save methods
which can be implemented with different combinations of pre-window
and post-window weights. This also allows experimentation with
non-rectangular windows."""

import math

import numpy as np

_REAL = np.float32    # numpy dtype used for real numbers
_COMPLEX = np.complex64   # numpy dtype used for complex numbers

def fcfb(input, freq, prewindow, weights, postwindow, overlap):
    """Apply a fast-convolution filter to a signal.

    Perform downconversion, filtering and resampling
    using the fast-convolution filter-bank method.

    FFT size is assumed to be even for now.
    Odd sizes might or might not work.

    Parameters
    ----------
    input
        Input signal as a numpy array.
    freq
        Center frequency of the down-converted channel.
        Unit is FFT bins.
    prewindow
        Window function for FFT as a numpy array.
        Length of this window determines the FFT size.
    weights
        Weights applied to frequency domain bins before IFFT.
        This determines the frequency response of the filter.
        Length determines the IFFT size.
        Bins are shifted so that:
        * weights[0] is the Nyquist frequency of the resampled signal
        * weights[1] is the most negative frequency
        * weights[ifft_size//2] is zero frequency
        * weights[ifft_size-1] is the most positive frequency
    postwindow
        Window function applied to the filtered signal after IFFT.
        Length must be equal to the IFFT size.
    overlap
        Overlap as a fraction of FFT size.
    """
    fft_size = len(prewindow)
    ifft_size = len(weights)

    input_step = int(round(fft_size * (1 - overlap)))
    ifft_overlap = int(round(ifft_size * overlap))

    outputs = []
    oo = None

    for t in range(0, len(input) - fft_size, input_step):
        # Windowed FFT of the input signal
        f = np.fft.fft(input[t : t+fft_size] * prewindow)
        # IFFT back to time domain.
        # FFT result is considered as circular, so use np.take in wrap mode
        # to pick the bins with proper handling of wrap-around
        # for all frequencies.
        o = np.fft.ifft(np.fft.fftshift(
            np.take(
                f,
                np.arange(freq - ifft_size//2, freq + ifft_size//2),
                mode='wrap'
            ) * weights
        )) * postwindow

        # Add the overlapping part from previous results
        if oo is not None:
            o[0 : ifft_overlap] += oo

        # Output the samples that will not be affected by overlap anymore
        outputs.append(o[0 : -ifft_overlap])
        # Store the rest of the samples to be overlapped and added
        oo = o[-ifft_overlap : ]

    return np.concatenate(outputs)


def zprect(l,n):
    """Return a zero-padded rectangular window
    having a total length of l, n ones in the middle
    and zero-padding at both ends"""
    w = np.zeros(l, dtype=_REAL)
    w[(l-n) // 2 : (l+n) // 2] = np.ones(n, dtype=_REAL)
    return w

def normalize_window(w):
    """Normalize the sum over a window to 1.
    This results in nicely scaled FFT result values."""
    return w / np.sum(w)

def fcfb_design(method='overlap_add'):
    """Design the parameters for a fast-convolution filter bank.

    Return the parameters as a dict that can be passed
    to fcfb function using **."""
    fft_size = 256
    ifft_size = 16
    overlap = 0.25
    if method == 'overlap_add':
        # Overlap-and-add: zero-padded prewindow, all-ones postwindow
        prewindow = zprect(fft_size, int(round(fft_size * (1-overlap))))
        postwindow = np.ones(ifft_size, dtype=_REAL)
    elif method == 'overlap_save':
        # Overlap-and-save: all-ones prewindow, zero-padded postwindow
        prewindow = np.ones(fft_size, dtype=_REAL)
        postwindow = zprect(ifft_size, int(round(ifft_size * (1-overlap))))

    # Make the weights symmetric around zero frequency.
    # Set the weight of the single bin at Nyquist frequency to zero.
    # Raised cosine filter:
    halfweights = 0.5+0.5*np.cos(np.linspace(0, math.pi, ifft_size//2, endpoint=False, dtype=_REAL))
    weights = np.concatenate(([0], halfweights[:0:-1], halfweights))

    return {
        'prewindow': normalize_window(prewindow),
        'weights': weights,
        'postwindow': postwindow,
        'overlap': overlap
    }


def complex_sine(f, l):
    """Return a complex sine wave of frequency f and length l."""
    phase = np.linspace(0, f*l, l, dtype=_REAL)
    return np.cos(phase) + 1j*np.sin(phase)

def power_db(s):
    """Return the power of a complex signal in dB."""
    try:
        return 10.0*math.log10(np.mean(s.real**2) + np.mean(s.imag**2))
    except ValueError:
        return -math.inf

def test():
    import matplotlib.pyplot as plt

    freqs = np.linspace(-np.pi, np.pi, 2**11+1)

    for method in ['overlap_add', 'overlap_save']:
        params = fcfb_design(method=method)
        # Test the filter with different input frequencies.
        # Let's start with a simple test
        # and just plot the total output power.
        # Discard some first samples of the result to let the output settle.
        results = [power_db(fcfb(complex_sine(f, 2**12), 50, **params)[32:]) for f in freqs]
        plt.plot(freqs, results)

    plt.xlabel('Input frequency (radian/s)')
    plt.ylabel('Output power (dB)')
    plt.legend(('Overlap and add', 'Overlap and save'))
    plt.grid()
    plt.show()

if __name__ == '__main__':
    test()
