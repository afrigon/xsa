import numpy as np

from operations.band_color import BandColor


def test_a_flat_spectrum_is_neutral_gray():
    color = BandColor([415, 566, 604, 643, 689]).linear_srgb(np.full(5, 0.3))

    assert np.allclose(color, 0.3, atol=1e-3)


def test_reflectance_rising_toward_red_is_warm():
    red, green, blue = BandColor([415, 566, 643]).linear_srgb(np.array([0.05, 0.1, 0.15]))

    assert red > green > blue
