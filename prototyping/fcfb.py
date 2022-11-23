#!/usr/bin/env python3
"""Implementation of a fast-convolution filter bank for testing purposes.

The implementation supports both overlap-add and overlap-save methods
which can be implemented with different combinations of pre-window
and post-window weights. This also allows experimentation with
non-rectangular windows."""

import math

import numpy as np
from scipy import signal

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
        # Numpy apparently normalizes IFFT results but we don't want that
        # since our input FFT window is normalized already. Undo normalization.
        o *= ifft_size

        # Add the overlapping part from previous results
        if oo is not None:
            o[0 : ifft_overlap] += oo

        # Output the samples that will not be affected by overlap anymore
        outputs.append(o[0 : -ifft_overlap])
        # Store the rest of the samples to be overlapped and added
        oo = o[-ifft_overlap : ]

    return np.concatenate(outputs)


def zptukey(l,n,t=0):
    """Return a zero-padded rectangular or Tukey window.
    The window has:
    * Total length of l
    * n non-zero bins in the middle
    * t bins of taper. If zero, make a rectangular window.
    """
    w = np.zeros(l, dtype=_REAL)
    w[(l-n) // 2 : (l+n) // 2] = signal.windows.tukey(n+2, (t+1)*2/(n+1))[1:-1]
    return w

def fcfb_design(method='overlap-add', taper=0, ifft_size=32, transition=16, shape='raisedcosine'):
    """Design the parameters for a fast-convolution filter bank.

    Return the parameters as a dict that can be passed
    to fcfb function using **."""
    fft_size = 256
    overlap = 0.25
    if method == 'overlap-add':
        # Overlap-and-add: zero-padded prewindow, all-ones postwindow
        prewindow = zptukey(fft_size, int(round(fft_size * (1-overlap) + taper)), taper)
        postwindow = np.ones(ifft_size, dtype=_REAL)
    elif method == 'overlap-save':
        # Overlap-and-save: all-ones prewindow, zero-padded postwindow
        #prewindow = np.ones(fft_size, dtype=_REAL)
        prewindow = zptukey(fft_size, fft_size, taper)
        postwindow = zptukey(ifft_size, int(round(ifft_size * (1-overlap))))
    print(prewindow)

    # Make the weights symmetric around zero frequency.
    # Set the weight of the single bin at Nyquist frequency to zero.
    if shape == 'raisedcosine':
        weights = signal.windows.tukey(ifft_size, 2.0*transition/ifft_size, sym=False)
    else:
        raise ValueError()
    print(weights)

    return {
        'prewindow': prewindow * (1.0/fft_size), # normalize
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

    freqs = np.linspace(-0.5, 0.5, 2**12+1)

    def plot_test(method, taper, ifft_size, transition):
        params = fcfb_design(method=method, taper=taper, ifft_size=ifft_size, transition=transition)
        # Test the filter with different input frequencies.
        # Let's start with a simple test
        # and just plot the total output power.
        # Discard some first samples of the result to let the output settle.
        results = [power_db(fcfb(complex_sine(f, 2**12), 0, **params)[ifft_size*2:]) for f in (freqs*(math.pi*2))]
        plt.plot(freqs, results)

    def figure():
        "Common code for all figures"
        plt.figure()
        plt.xlabel('Input frequency (1/sample rate)')
        plt.ylabel('Output power (dB)')
        plt.xlim([-0.5, 0.5])
        plt.ylim([-120, 3])
        plt.grid()

    def plot_oa_vs_os(ifft_size=32, taper=0):
        "Comparison of overlap-add and overlap-save methods"
        figure()
        legends = list()
        for method in ['overlap-save', 'overlap-add']:
            plot_test(method, taper, ifft_size, ifft_size//2)
            legends.append("%s" % (method, ))
        plt.legend(legends)

    def plot_weights(taper=0, method='overlap-save'):
        "Effect of IFFT size and transition band width"
        figure()
        legends = list()
        for ifft_size, transition in ((32, 16), (64, 16), (64, 32), (128, 16), (128, 64)):
            plot_test(method, taper, ifft_size, transition)
            legends.append("IFFT size=%d, transition band %d bins" % (ifft_size, transition))
        plt.legend(legends)

    def plot_taper():
        "Effect of input window tapering"
        figure()
        legends = list()
        for method in ('overlap-save', 'overlap-add'):
            for taper in (0, 32):
                plot_test(method, taper, 32, 8)
                legends.append("%s, taper %d samples" % (method, taper))
        plt.legend(legends)

    plot_weights()
    plot_oa_vs_os()
    plot_taper()
    plt.show()

if __name__ == '__main__':
    test()
