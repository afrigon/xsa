import numpy as np

from operations.color_grade import LUMINANCE_WEIGHTS, ColorGrade
from operations.raster import Raster


def test_luminance_is_kept_and_color_moves_toward_gray():
    color = Raster(np.array([[[0.11, 0.09, 0.07]]], np.float32))
    graded = ColorGrade(0.2).apply(color).pixels[0, 0]
    original = color.pixels[0, 0]

    assert np.isclose(graded @ LUMINANCE_WEIGHTS, original @ LUMINANCE_WEIGHTS, atol=1e-6)
    assert graded[0] / graded[2] < original[0] / original[2]


def test_full_saturation_changes_nothing():
    color = Raster(np.random.default_rng(4).random((2, 4, 3)).astype(np.float32))
    assert np.allclose(ColorGrade(1.0).apply(color).pixels, color.pixels)
