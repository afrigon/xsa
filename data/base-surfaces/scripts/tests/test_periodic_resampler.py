import numpy as np

from operations.periodic_resampler import PeriodicResampler


def wave(size: int, cycles: int) -> np.ndarray:
    x = (np.arange(size) + 0.5) / size
    row = np.sin(2 * np.pi * cycles * x)
    return (row[None, :] + row[:, None])[..., None]


def test_a_tiling_wave_stays_the_same_wave_across_the_seam():
    resampled = PeriodicResampler(300, 300).resample(wave(512, 5))

    assert np.abs(resampled - wave(300, 5)).max() < 1e-3


def test_resampling_keeps_the_mean():
    pixels = np.random.default_rng(1).random((256, 256, 3))
    resampled = PeriodicResampler(100, 100).resample(pixels)

    assert np.allclose(resampled.mean(axis=(0, 1)), pixels.mean(axis=(0, 1)), atol=1e-4)


def test_the_same_size_is_left_unchanged():
    pixels = np.random.default_rng(2).random((64, 64, 1))

    assert np.allclose(PeriodicResampler(64, 64).resample(pixels), pixels, atol=1e-6)
